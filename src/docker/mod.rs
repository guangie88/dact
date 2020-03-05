pub mod conf;
mod fmt;

pub use conf::{Action, Actions, Conf, Help, Run};
use std::path::PathBuf;
use which::which;

pub fn get_cli_path() -> Result<PathBuf, which::Error> {
    which("docker")
}
