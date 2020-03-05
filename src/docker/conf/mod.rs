pub mod v1_0;

use crate::docker::fmt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub enum Action {
    V1_0(v1_0::Action),
}

pub trait Help {
    fn help(&self) -> Option<String>;
}

pub trait Run {
    fn run(
        &self,
        docker_cmd: &Path,
        envs: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>>;
}

pub fn shell_interpolate(raw: &str) -> Result<String, Box<dyn Error>> {
    fmt::shell_interpolate(raw, &|cmd| {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", cmd]).output()?
        } else {
            Command::new("sh").args(&["-c", cmd]).output()?
        };

        Ok(std::str::from_utf8(&output.stdout)?.trim_end().to_string())
    })
}

impl Help for Action {
    fn help(&self) -> Option<String> {
        use Action::*;

        match self {
            V1_0(action) => action.help(),
        }
    }
}

impl Run for Action {
    fn run(
        &self,
        docker_cmd: &Path,
        envs: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        use Action::*;

        match self {
            V1_0(action) => action.run(docker_cmd, envs),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Conf {
    V1_0(v1_0::Conf),
}

pub trait Actions {
    fn actions(&self) -> HashMap<String, Action>;
}

impl Actions for Conf {
    fn actions(&self) -> HashMap<String, Action> {
        match self {
            Conf::V1_0(conf) => conf.actions(),
        }
    }
}
