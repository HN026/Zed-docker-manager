//! Docker Compose service management commands.

use crate::docker::{self, DockerExecutor};
use crate::types::CommandOutput;

// Starts compose services. Common flags include -d (detached) and
// --build (rebuild images before starting).
pub fn up(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut docker_args = vec!["compose", "up"];
    docker_args.extend_from_slice(&str_args);
    let text = docker::run_docker_command(executor, &docker_args)?;
    Ok(CommandOutput::new("Docker Compose Up", text))
}

// Stops and removes compose services, networks, and optionally volumes.
pub fn down(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut docker_args = vec!["compose", "down"];
    docker_args.extend_from_slice(&str_args);
    let text = docker::run_docker_command(executor, &docker_args)?;
    Ok(CommandOutput::new("Docker Compose Down", text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockDockerExecutor;

    #[test]
    fn up_calls_compose_up() {
        let executor = MockDockerExecutor::with_success("Creating network...");
        let result = up(&executor, &[]).unwrap();
        assert_eq!(result.label, "Docker Compose Up");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["compose", "up"]);
    }

    #[test]
    fn up_with_detached_flag() {
        let executor = MockDockerExecutor::with_success("Starting services...");
        let _ = up(&executor, &["-d".to_string()]);

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["compose", "up", "-d"]);
    }

    #[test]
    fn up_with_build_flag() {
        let executor = MockDockerExecutor::with_success("Building...");
        let _ = up(&executor, &["--build".to_string()]);

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["compose", "up", "--build"]);
    }

    #[test]
    fn up_with_multiple_flags() {
        let executor = MockDockerExecutor::with_success("ok");
        let _ = up(&executor, &["-d".to_string(), "--build".to_string()]);

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["compose", "up", "-d", "--build"]);
    }

    #[test]
    fn down_calls_compose_down() {
        let executor = MockDockerExecutor::with_success("Stopping services...");
        let result = down(&executor, &[]).unwrap();
        assert_eq!(result.label, "Docker Compose Down");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["compose", "down"]);
    }

    #[test]
    fn down_with_volumes_flag() {
        let executor = MockDockerExecutor::with_success("Removing volumes...");
        let _ = down(&executor, &["-v".to_string()]);

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["compose", "down", "-v"]);
    }

    #[test]
    fn commands_propagate_docker_errors() {
        let executor = MockDockerExecutor::with_error("no compose file found");
        assert!(up(&executor, &[]).is_err());
        assert!(down(&executor, &[]).is_err());
    }
}
