//! Image management commands including listing, removal, pulling, running,
//! and building.

use crate::docker::{self, DockerExecutor};
use crate::types::CommandOutput;

// Lists available Docker images with repository, tag, ID, size, and age.
pub fn list(executor: &dyn DockerExecutor, _args: &[String]) -> Result<CommandOutput, String> {
    let text = docker::run_docker_command(
        executor,
        &[
            "images",
            "--format",
            "table {{.Repository}}\t{{.Tag}}\t{{.ID}}\t{{.Size}}\t{{.CreatedSince}}",
        ],
    )?;
    Ok(CommandOutput::new("Docker Images", text))
}

// Removes a Docker image by name or ID.
pub fn rmi(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let image = args
        .first()
        .ok_or_else(|| "Please provide an image name or ID.".to_string())?;
    let text = docker::run_docker_command(executor, &["rmi", image])?;
    Ok(CommandOutput::new(format!("Removed Image: {image}"), text))
}

// Pulls an image from a container registry.
pub fn pull(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let image = args
        .first()
        .ok_or_else(|| "Please provide an image name (e.g. 'nginx:latest').".to_string())?;
    let text = docker::run_docker_command(executor, &["pull", image])?;
    Ok(CommandOutput::new(format!("Pulled: {image}"), text))
}

// Runs a new container from an image. All arguments are forwarded to docker run.
pub fn run(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    if args.is_empty() {
        return Err(
            "Please provide an image name (e.g. 'ubuntu:latest --name my-container -d')."
                .to_string(),
        );
    }
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut docker_args = vec!["run"];
    docker_args.extend_from_slice(&str_args);
    let text = docker::run_docker_command(executor, &docker_args)?;
    Ok(CommandOutput::new("Docker Run", text))
}

// Builds an image from a Dockerfile. All arguments are forwarded to docker build.
pub fn build(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    if args.is_empty() {
        return Err(
            "Usage: /docker-build <args>\nExample: /docker-build -t my-image .".to_string(),
        );
    }
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut docker_args = vec!["build"];
    docker_args.extend_from_slice(&str_args);
    let text = docker::run_docker_command(executor, &docker_args)?;
    Ok(CommandOutput::new("Docker Build", text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockDockerExecutor;

    #[test]
    fn list_calls_docker_images_with_format() {
        let executor = MockDockerExecutor::with_success("REPO  TAG  ID  SIZE  CREATED");
        let result = list(&executor, &[]).unwrap();
        assert_eq!(result.label, "Docker Images");

        let captured = executor.captured_args();
        assert_eq!(captured[0][0], "images");
        assert!(captured[0].contains(&"--format".to_string()));
    }

    #[test]
    fn rmi_removes_image() {
        let executor = MockDockerExecutor::with_success("Untagged: nginx:latest");
        let result = rmi(&executor, &["nginx:latest".to_string()]).unwrap();
        assert_eq!(result.label, "Removed Image: nginx:latest");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["rmi", "nginx:latest"]);
    }

    #[test]
    fn rmi_errors_without_image_name() {
        let executor = MockDockerExecutor::with_success("");
        let result = rmi(&executor, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("image name or ID"));
    }

    #[test]
    fn pull_pulls_image() {
        let executor = MockDockerExecutor::with_success("Pulling from library/nginx");
        let result = pull(&executor, &["nginx:latest".to_string()]).unwrap();
        assert_eq!(result.label, "Pulled: nginx:latest");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["pull", "nginx:latest"]);
    }

    #[test]
    fn pull_errors_without_image_name() {
        let executor = MockDockerExecutor::with_success("");
        let result = pull(&executor, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn run_creates_container_with_flags() {
        let executor = MockDockerExecutor::with_success("abc123");
        let result = run(
            &executor,
            &[
                "-d".to_string(),
                "--name".to_string(),
                "web".to_string(),
                "nginx:latest".to_string(),
            ],
        )
        .unwrap();
        assert_eq!(result.label, "Docker Run");

        let captured = executor.captured_args();
        assert_eq!(
            captured[0],
            vec!["run", "-d", "--name", "web", "nginx:latest"]
        );
    }

    #[test]
    fn run_errors_without_image() {
        let executor = MockDockerExecutor::with_success("");
        let result = run(&executor, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("image name"));
    }

    #[test]
    fn build_builds_image_with_tag() {
        let executor = MockDockerExecutor::with_success("Successfully built abc123");
        let result = build(
            &executor,
            &["-t".to_string(), "my-app".to_string(), ".".to_string()],
        )
        .unwrap();
        assert_eq!(result.label, "Docker Build");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["build", "-t", "my-app", "."]);
    }

    #[test]
    fn build_errors_without_args() {
        let executor = MockDockerExecutor::with_success("");
        let result = build(&executor, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn commands_propagate_docker_errors() {
        let executor = MockDockerExecutor::with_error("daemon not running");

        assert!(list(&executor, &[]).is_err());
        assert!(rmi(&executor, &["img".to_string()]).is_err());
        assert!(pull(&executor, &["img".to_string()]).is_err());
        assert!(run(&executor, &["img".to_string()]).is_err());
        assert!(build(&executor, &["-t".to_string(), "x".to_string(), ".".to_string()]).is_err());
    }
}
