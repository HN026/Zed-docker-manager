//! Container lifecycle and inspection commands.

use crate::docker::{self, DockerExecutor};
use crate::types::CommandOutput;

// Lists containers. Accepts "all", "-a", or "--all" to include stopped containers.
pub fn ps(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let show_all = args
        .iter()
        .any(|a| a == "all" || a == "-a" || a == "--all");

    let mut docker_args = vec![
        "ps",
        "--format",
        "table {{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}\t{{.Ports}}",
    ];
    if show_all {
        docker_args.push("-a");
    }

    let text = docker::run_docker_command(executor, &docker_args)?;
    let label = if show_all {
        "Docker Containers (all)"
    } else {
        "Docker Containers (running)"
    };
    Ok(CommandOutput::new(label, text))
}

// Starts a stopped container by name or ID.
pub fn start(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let container = args
        .first()
        .ok_or_else(|| "Please provide a container name or ID.".to_string())?;
    let text = docker::run_docker_command(executor, &["start", container])?;
    Ok(CommandOutput::new(format!("Started: {container}"), text))
}

// Stops a running container by name or ID.
pub fn stop(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let container = args
        .first()
        .ok_or_else(|| "Please provide a container name or ID.".to_string())?;
    let text = docker::run_docker_command(executor, &["stop", container])?;
    Ok(CommandOutput::new(format!("Stopped: {container}"), text))
}

// Removes a container. All arguments including flags like --force are forwarded.
pub fn rm(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    if args.is_empty() {
        return Err("Please provide a container name or ID.".to_string());
    }
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut docker_args = vec!["rm"];
    docker_args.extend_from_slice(&str_args);
    let text = docker::run_docker_command(executor, &docker_args)?;
    Ok(CommandOutput::new("Removed Container", text))
}

// Fetches logs for a container. Supports additional flags like --tail N.
pub fn logs(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    if args.is_empty() {
        return Err("Please provide a container name or ID.".to_string());
    }
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut docker_args = vec!["logs"];
    docker_args.extend_from_slice(&str_args);
    let text = docker::run_docker_command(executor, &docker_args)?;
    let container = &args[0];
    Ok(CommandOutput::new(format!("Logs: {container}"), text))
}

// Shows a point-in-time snapshot of container resource usage.
pub fn stats(executor: &dyn DockerExecutor, _args: &[String]) -> Result<CommandOutput, String> {
    let text = docker::run_docker_command(
        executor,
        &[
            "stats",
            "--no-stream",
            "--format",
            "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}\t{{.NetIO}}\t{{.BlockIO}}",
        ],
    )?;
    Ok(CommandOutput::new("Docker Stats", text))
}

// Returns the full JSON inspection output for a container.
pub fn inspect(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    let container = args
        .first()
        .ok_or_else(|| "Please provide a container name or ID.".to_string())?;
    let text = docker::run_docker_command(executor, &["inspect", container])?;
    Ok(CommandOutput::new(format!("Inspect: {container}"), text))
}

// Executes a command inside a running container. Requires at least a
// container name and a command to run.
pub fn exec(executor: &dyn DockerExecutor, args: &[String]) -> Result<CommandOutput, String> {
    if args.len() < 2 {
        return Err(
            "Usage: /docker-exec <container> <command> [args...]\n\
             Example: /docker-exec my-container ls -la"
                .to_string(),
        );
    }
    let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut docker_args = vec!["exec"];
    docker_args.extend_from_slice(&str_args);
    let text = docker::run_docker_command(executor, &docker_args)?;
    let container = &args[0];
    Ok(CommandOutput::new(format!("Exec in: {container}"), text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::MockDockerExecutor;

    #[test]
    fn ps_without_args_shows_running_containers() {
        let executor = MockDockerExecutor::with_success("ID  NAME  IMAGE  STATUS  PORTS");
        let result = ps(&executor, &[]).unwrap();
        assert_eq!(result.label, "Docker Containers (running)");
        assert!(result.text.contains("NAME"));

        let captured = executor.captured_args();
        assert_eq!(captured.len(), 1);
        assert!(!captured[0].contains(&"-a".to_string()));
    }

    #[test]
    fn ps_with_all_flag_includes_stopped() {
        let executor = MockDockerExecutor::with_success("ID  NAME  IMAGE  STATUS  PORTS");
        let result = ps(&executor, &["all".to_string()]).unwrap();
        assert_eq!(result.label, "Docker Containers (all)");

        let captured = executor.captured_args();
        assert!(captured[0].contains(&"-a".to_string()));
    }

    #[test]
    fn ps_with_dash_a_includes_stopped() {
        let executor = MockDockerExecutor::with_success("ok");
        let result = ps(&executor, &["-a".to_string()]).unwrap();
        assert_eq!(result.label, "Docker Containers (all)");
    }

    #[test]
    fn ps_with_double_dash_all_includes_stopped() {
        let executor = MockDockerExecutor::with_success("ok");
        let result = ps(&executor, &["--all".to_string()]).unwrap();
        assert_eq!(result.label, "Docker Containers (all)");
    }

    #[test]
    fn start_calls_docker_start_with_container_name() {
        let executor = MockDockerExecutor::with_success("my-container");
        let result = start(&executor, &["my-container".to_string()]).unwrap();
        assert_eq!(result.label, "Started: my-container");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["start", "my-container"]);
    }

    #[test]
    fn start_errors_without_container_name() {
        let executor = MockDockerExecutor::with_success("");
        let result = start(&executor, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("container name or ID"));
    }

    #[test]
    fn stop_calls_docker_stop_with_container_name() {
        let executor = MockDockerExecutor::with_success("my-container");
        let result = stop(&executor, &["my-container".to_string()]).unwrap();
        assert_eq!(result.label, "Stopped: my-container");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["stop", "my-container"]);
    }

    #[test]
    fn stop_errors_without_container_name() {
        let executor = MockDockerExecutor::with_success("");
        let result = stop(&executor, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn rm_removes_container() {
        let executor = MockDockerExecutor::with_success("removed");
        let result = rm(&executor, &["old-container".to_string()]).unwrap();
        assert_eq!(result.label, "Removed Container");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["rm", "old-container"]);
    }

    #[test]
    fn rm_with_force_flag() {
        let executor = MockDockerExecutor::with_success("removed");
        let result = rm(
            &executor,
            &["--force".to_string(), "old-container".to_string()],
        )
        .unwrap();
        assert_eq!(result.label, "Removed Container");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["rm", "--force", "old-container"]);
    }

    #[test]
    fn rm_errors_without_container_name() {
        let executor = MockDockerExecutor::with_success("");
        let result = rm(&executor, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("container name or ID"));
    }

    #[test]
    fn logs_fetches_container_logs() {
        let executor = MockDockerExecutor::with_success("log line 1\nlog line 2");
        let result = logs(&executor, &["web".to_string()]).unwrap();
        assert_eq!(result.label, "Logs: web");
        assert!(result.text.contains("log line 1"));
    }

    #[test]
    fn logs_with_tail_flag() {
        let executor = MockDockerExecutor::with_success("last line");
        let _ = logs(
            &executor,
            &["web".to_string(), "--tail".to_string(), "10".to_string()],
        );

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["logs", "web", "--tail", "10"]);
    }

    #[test]
    fn logs_errors_without_container() {
        let executor = MockDockerExecutor::with_success("");
        let result = logs(&executor, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn stats_calls_docker_stats_no_stream() {
        let executor = MockDockerExecutor::with_success("NAME  CPU  MEM");
        let result = stats(&executor, &[]).unwrap();
        assert_eq!(result.label, "Docker Stats");

        let captured = executor.captured_args();
        assert!(captured[0].contains(&"--no-stream".to_string()));
    }

    #[test]
    fn inspect_fetches_container_config() {
        let executor = MockDockerExecutor::with_success("{\"Id\": \"abc123\"}");
        let result = inspect(&executor, &["web".to_string()]).unwrap();
        assert_eq!(result.label, "Inspect: web");
        assert!(result.text.contains("abc123"));
    }

    #[test]
    fn inspect_errors_without_container() {
        let executor = MockDockerExecutor::with_success("");
        let result = inspect(&executor, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn exec_runs_command_in_container() {
        let executor = MockDockerExecutor::with_success("file1\nfile2");
        let result = exec(
            &executor,
            &["web".to_string(), "ls".to_string(), "-la".to_string()],
        )
        .unwrap();
        assert_eq!(result.label, "Exec in: web");

        let captured = executor.captured_args();
        assert_eq!(captured[0], vec!["exec", "web", "ls", "-la"]);
    }

    #[test]
    fn exec_errors_with_only_container_name() {
        let executor = MockDockerExecutor::with_success("");
        let result = exec(&executor, &["web".to_string()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Usage"));
    }

    #[test]
    fn exec_errors_with_no_args() {
        let executor = MockDockerExecutor::with_success("");
        let result = exec(&executor, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn commands_propagate_docker_errors() {
        let executor = MockDockerExecutor::with_error("daemon not running");

        assert!(ps(&executor, &[]).is_err());
        assert!(start(&executor, &["c".to_string()]).is_err());
        assert!(stop(&executor, &["c".to_string()]).is_err());
        assert!(rm(&executor, &["c".to_string()]).is_err());
        assert!(logs(&executor, &["c".to_string()]).is_err());
        assert!(stats(&executor, &[]).is_err());
        assert!(inspect(&executor, &["c".to_string()]).is_err());
        assert!(exec(&executor, &["c".to_string(), "ls".to_string()]).is_err());
    }
}
