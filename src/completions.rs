//! Dynamic argument completion for Docker slash commands. Queries the
//! Docker daemon at completion time to suggest container names, image
//! names, and common flags.

use crate::docker::DockerExecutor;

// A single completion entry for a slash command argument.
pub struct Completion {
    pub label: String,
    pub new_text: String,
    pub run_command: bool,
}

impl Completion {
    // Creates a completion that runs the command immediately on acceptance.
    pub fn auto_run(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            new_text: text.into(),
            run_command: true,
        }
    }

    // Creates a completion that inserts text without running the command.
    pub fn insert_only(label: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            new_text: text.into(),
            run_command: false,
        }
    }
}

// Returns names of currently running containers.
pub fn list_container_names(executor: &dyn DockerExecutor) -> Vec<String> {
    match executor.execute(&["ps", "--format", "{{.Names}}"]) {
        Ok(output) => parse_lines(&output),
        Err(_) => vec![],
    }
}

// Returns names of all containers including stopped ones.
pub fn list_all_container_names(executor: &dyn DockerExecutor) -> Vec<String> {
    match executor.execute(&["ps", "-a", "--format", "{{.Names}}"]) {
        Ok(output) => parse_lines(&output),
        Err(_) => vec![],
    }
}

// Returns image names as repository:tag, filtering out untagged images.
pub fn list_image_names(executor: &dyn DockerExecutor) -> Vec<String> {
    match executor.execute(&["images", "--format", "{{.Repository}}:{{.Tag}}"]) {
        Ok(output) => parse_lines(&output)
            .into_iter()
            .filter(|name| name != "<none>:<none>")
            .collect(),
        Err(_) => vec![],
    }
}

// Generates completions appropriate for the given slash command name.
pub fn complete(
    command_name: &str,
    executor: &dyn DockerExecutor,
) -> Result<Vec<Completion>, String> {
    match command_name {
        "docker-stop" | "docker-logs" | "docker-exec" | "docker-inspect" | "docker-stats" => {
            Ok(list_container_names(executor)
                .into_iter()
                .map(|name| Completion::auto_run(name.clone(), name))
                .collect())
        }

        "docker-start" | "docker-rm" => Ok(list_all_container_names(executor)
            .into_iter()
            .map(|name| Completion::auto_run(name.clone(), name))
            .collect()),

        "docker-rmi" => Ok(list_image_names(executor)
            .into_iter()
            .map(|name| Completion::auto_run(name.clone(), name))
            .collect()),

        "docker-run" | "docker-pull" => Ok(list_image_names(executor)
            .into_iter()
            .map(|name| Completion::insert_only(name.clone(), name))
            .collect()),

        "docker-ps" => Ok(vec![Completion::auto_run("all", "all")]),

        "docker-compose-up" => Ok(vec![
            Completion::auto_run("Detached mode (-d)", "-d"),
            Completion::auto_run("Build before starting (--build)", "--build"),
        ]),

        "docker-build" => Ok(vec![Completion::insert_only(
            "Build current directory",
            "-t my-image .",
        )]),

        _ => Ok(vec![]),
    }
}

// Splits newline-delimited output into trimmed, non-empty lines.
fn parse_lines(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockDockerExecutor;

    #[test]
    fn parse_lines_splits_on_newlines() {
        let result = parse_lines("alpha\nbeta\ngamma");
        assert_eq!(result, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn parse_lines_trims_whitespace() {
        let result = parse_lines("  alpha  \n  beta  ");
        assert_eq!(result, vec!["alpha", "beta"]);
    }

    #[test]
    fn parse_lines_filters_empty_lines() {
        let result = parse_lines("alpha\n\n\nbeta\n");
        assert_eq!(result, vec!["alpha", "beta"]);
    }

    #[test]
    fn parse_lines_empty_input() {
        let result = parse_lines("");
        assert!(result.is_empty());
    }

    #[test]
    fn list_container_names_returns_names() {
        let executor = MockDockerExecutor::with_success("web\ndb\ncache");
        let names = list_container_names(&executor);
        assert_eq!(names, vec!["web", "db", "cache"]);
    }

    #[test]
    fn list_container_names_returns_empty_on_error() {
        let executor = MockDockerExecutor::with_error("docker not found");
        let names = list_container_names(&executor);
        assert!(names.is_empty());
    }

    #[test]
    fn list_all_container_names_includes_stopped() {
        let executor = MockDockerExecutor::with_success("web\nstopped-db");
        let names = list_all_container_names(&executor);
        assert_eq!(names, vec!["web", "stopped-db"]);

        let captured = executor.captured_args();
        assert!(captured[0].contains(&"-a".to_string()));
    }

    #[test]
    fn list_image_names_filters_none() {
        let executor =
            MockDockerExecutor::with_success("nginx:latest\n<none>:<none>\nubuntu:22.04");
        let names = list_image_names(&executor);
        assert_eq!(names, vec!["nginx:latest", "ubuntu:22.04"]);
    }

    #[test]
    fn list_image_names_returns_empty_on_error() {
        let executor = MockDockerExecutor::with_error("docker not found");
        let names = list_image_names(&executor);
        assert!(names.is_empty());
    }

    #[test]
    fn complete_docker_stop_returns_running_containers() {
        let executor = MockDockerExecutor::with_success("web\ndb");
        let completions = complete("docker-stop", &executor).unwrap();
        assert_eq!(completions.len(), 2);
        assert_eq!(completions[0].label, "web");
        assert!(completions[0].run_command);
    }

    #[test]
    fn complete_docker_start_returns_all_containers() {
        let executor = MockDockerExecutor::with_success("web\nstopped-svc");
        let completions = complete("docker-start", &executor).unwrap();
        assert_eq!(completions.len(), 2);

        let captured = executor.captured_args();
        assert!(captured[0].contains(&"-a".to_string()));
    }

    #[test]
    fn complete_docker_run_returns_images_insert_only() {
        let executor = MockDockerExecutor::with_success("nginx:latest\nubuntu:22.04");
        let completions = complete("docker-run", &executor).unwrap();
        assert_eq!(completions.len(), 2);
        assert!(!completions[0].run_command);
    }

    #[test]
    fn complete_docker_rmi_returns_images_auto_run() {
        let executor = MockDockerExecutor::with_success("nginx:latest");
        let completions = complete("docker-rmi", &executor).unwrap();
        assert_eq!(completions.len(), 1);
        assert!(completions[0].run_command);
    }

    #[test]
    fn complete_docker_ps_returns_all_option() {
        let executor = MockDockerExecutor::with_success("");
        let completions = complete("docker-ps", &executor).unwrap();
        assert_eq!(completions.len(), 1);
        assert_eq!(completions[0].new_text, "all");
    }

    #[test]
    fn complete_docker_compose_up_returns_flags() {
        let executor = MockDockerExecutor::with_success("");
        let completions = complete("docker-compose-up", &executor).unwrap();
        assert_eq!(completions.len(), 2);
        assert_eq!(completions[0].new_text, "-d");
        assert_eq!(completions[1].new_text, "--build");
    }

    #[test]
    fn complete_unknown_command_returns_empty() {
        let executor = MockDockerExecutor::with_success("");
        let completions = complete("unknown", &executor).unwrap();
        assert!(completions.is_empty());
    }
}
