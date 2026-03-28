//! Zed extension glue layer. Bridges between the Zed API types and the
//! pure-logic modules. Only compiled when targeting WebAssembly.

use zed_extension_api::{
    self as zed, SlashCommand, SlashCommandArgumentCompletion, SlashCommandOutput,
    SlashCommandOutputSection, Worktree,
};

use crate::commands::{compose, containers, images};
use crate::completions;
use crate::docker::DockerExecutor;
use crate::types::CommandOutput;

// The Zed extension entry point.
pub struct DockerManagerExtension;

// Docker executor backed by the Zed process API.
struct ZedDockerExecutor;

impl DockerExecutor for ZedDockerExecutor {
    fn execute(&self, args: &[&str]) -> Result<String, String> {
        let output = zed::process::Command::new("docker")
            .args(args.iter().map(|s| s.to_string()))
            .output()
            .map_err(|e| format!("Failed to execute docker command: {e}"))?;

        let success = match output.status {
            Some(code) => code == 0,
            None => true,
        };

        if success {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if stderr.trim().is_empty() {
                Err("Docker command failed with no error message.".to_string())
            } else {
                Err(format!("Docker error: {stderr}"))
            }
        }
    }
}

impl DockerManagerExtension {
    fn to_slash_output(output: CommandOutput) -> SlashCommandOutput {
        let len = output.text.len();
        SlashCommandOutput {
            sections: vec![SlashCommandOutputSection {
                range: (0..len).into(),
                label: output.label,
            }],
            text: output.text,
        }
    }

    fn to_slash_completion(c: completions::Completion) -> SlashCommandArgumentCompletion {
        SlashCommandArgumentCompletion {
            label: c.label,
            new_text: c.new_text,
            run_command: c.run_command,
        }
    }
}

impl zed::Extension for DockerManagerExtension {
    fn new() -> Self {
        DockerManagerExtension
    }

    fn run_slash_command(
        &self,
        command: SlashCommand,
        args: Vec<String>,
        _worktree: Option<&Worktree>,
    ) -> Result<SlashCommandOutput, String> {
        let executor = ZedDockerExecutor;

        let result = match command.name.as_str() {
            "docker-ps" => containers::ps(&executor, &args),
            "docker-start" => containers::start(&executor, &args),
            "docker-stop" => containers::stop(&executor, &args),
            "docker-rm" => containers::rm(&executor, &args),
            "docker-logs" => containers::logs(&executor, &args),
            "docker-stats" => containers::stats(&executor, &args),
            "docker-inspect" => containers::inspect(&executor, &args),
            "docker-exec" => containers::exec(&executor, &args),
            "docker-images" => images::list(&executor, &args),
            "docker-rmi" => images::rmi(&executor, &args),
            "docker-pull" => images::pull(&executor, &args),
            "docker-run" => images::run(&executor, &args),
            "docker-build" => images::build(&executor, &args),
            "docker-compose-up" => compose::up(&executor, &args),
            "docker-compose-down" => compose::down(&executor, &args),
            cmd => Err(format!("Unknown Docker command: \"{cmd}\"")),
        }?;

        Ok(Self::to_slash_output(result))
    }

    fn complete_slash_command_argument(
        &self,
        command: SlashCommand,
        _args: Vec<String>,
    ) -> Result<Vec<SlashCommandArgumentCompletion>, String> {
        let executor = ZedDockerExecutor;
        let items = completions::complete(&command.name, &executor)?;
        Ok(items
            .into_iter()
            .map(Self::to_slash_completion)
            .collect())
    }
}

zed::register_extension!(DockerManagerExtension);
