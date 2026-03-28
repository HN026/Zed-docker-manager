//! Abstraction over Docker CLI execution, enabling mock-based testing.

// Trait representing the ability to execute Docker CLI commands.
pub trait DockerExecutor {
    // Executes a Docker CLI command with the given arguments.
    // Returns stdout on success or an error message on failure.
    fn execute(&self, args: &[&str]) -> Result<String, String>;
}

// Executes a Docker command through the provided executor, normalizing
// empty output into a human-readable success message.
pub fn run_docker_command(
    executor: &dyn DockerExecutor,
    args: &[&str],
) -> Result<String, String> {
    let output = executor.execute(args)?;
    if output.trim().is_empty() {
        Ok("Command completed successfully (no output).".to_string())
    } else {
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockDockerExecutor;

    #[test]
    fn run_docker_command_returns_output_on_success() {
        let executor = MockDockerExecutor::with_success("container1\ncontainer2");
        let result = run_docker_command(&executor, &["ps"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "container1\ncontainer2");
    }

    #[test]
    fn run_docker_command_replaces_empty_output() {
        let executor = MockDockerExecutor::with_success("");
        let result = run_docker_command(&executor, &["stop", "abc"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Command completed successfully (no output).");
    }

    #[test]
    fn run_docker_command_replaces_whitespace_only_output() {
        let executor = MockDockerExecutor::with_success("   \n  ");
        let result = run_docker_command(&executor, &["rm", "abc"]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Command completed successfully (no output).");
    }

    #[test]
    fn run_docker_command_propagates_error() {
        let executor = MockDockerExecutor::with_error("container not found");
        let result = run_docker_command(&executor, &["stop", "missing"]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "container not found");
    }

    #[test]
    fn run_docker_command_captures_args() {
        let executor = MockDockerExecutor::with_success("ok");
        let _ = run_docker_command(&executor, &["ps", "-a", "--format", "table"]);
        let captured = executor.captured_args();
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0], vec!["ps", "-a", "--format", "table"]);
    }
}
