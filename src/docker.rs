use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use which::which;

#[derive(Debug)]
pub struct DockerRun {
    envs: HashMap<String, String>,
    env_file: PathBuf,
    interactive: bool,
}

impl DockerRun {
    pub fn run() {}
}

pub fn get_docker_cli_path() -> Result<PathBuf, which::Error> {
    which("docker")
}
