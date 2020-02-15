use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerRun {
    pub image: String,
    pub help: Option<String>,
    pub command: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub envs: Option<HashMap<String, String>>,
    pub env_file: Option<PathBuf>,
}

impl DockerRun {
    pub fn run(&self, docker_cmd: &Path) -> Result<(), Box<dyn Error>> {
        // Convert all options into flags
        let command_flags =
            self.command.as_ref().map_or(vec![], |cmds| cmds.clone());

        let env_flags = self.envs.as_ref().map_or(vec![], |envs| {
            envs.iter()
                .map(|(k, v)| format!("-e {}={}", k, v))
                .collect()
        });

        let env_file_flags =
            self.env_file.as_ref().map_or(vec![], |env_file| {
                vec![format!("--env-file {}", env_file.display())]
            });

        let args = [
            &["run".to_string()],
            &["--rm".to_string()],
            &env_flags[..],
            &env_file_flags[..],
            &[self.image.clone()],
            &command_flags[..],
        ]
        .concat();

        let output = Command::new(docker_cmd).args(args).output()?;

        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
        Ok(())
    }
}

pub fn get_cli_path() -> Result<PathBuf, which::Error> {
    which("docker")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dockerrun(image: &str) -> DockerRun {
        DockerRun {
            image: image.to_string(),
            help: None,
            command: None,
            entrypoint: None,
            envs: None,
            env_file: None,
        }
    }

    #[test]
    fn test_run() {
        let mut dr = make_dockerrun("clux/muslrust:stable");
        dr.command = Some(vec!["cargo".to_string(), "--version".to_string()]);

        let docker_cmd = get_cli_path().unwrap();
        dr.run(&docker_cmd).unwrap();
    }
}
