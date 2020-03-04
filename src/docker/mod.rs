use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::default::Default;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use which::which;

mod fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Context {
    Image(String),
    Build(String),
}

impl Default for Context {
    fn default() -> Self {
        Context::Image("".to_string())
    }
}

#[derive(Debug, Default, Builder, Serialize, Deserialize)]
#[builder(setter(into))]
pub struct DockerRun {
    #[serde(flatten)]
    #[builder()]
    pub context: Context,

    #[builder(default = "None")]
    pub help: Option<String>,

    #[builder(default = "None")]
    pub interactive: Option<bool>,

    #[builder(default = "None")]
    pub tty: Option<bool>,

    #[builder(default = "None")]
    pub command: Option<Vec<String>>,

    #[builder(default = "None")]
    pub entrypoint: Option<String>,

    #[builder(default = "None")]
    pub envs: Option<HashMap<String, String>>,

    #[builder(default = "None")]
    pub env_file: Option<PathBuf>,

    #[builder(default = "None")]
    pub network: Option<String>,

    #[builder(default = "None")]
    pub ports: Option<Vec<String>>,

    #[builder(default = "None")]
    pub volumes: Option<Vec<String>>,

    #[builder(default = "None")]
    pub user: Option<String>,

    #[builder(default = "None")]
    pub extra_flags: Option<Vec<String>>,
}

pub fn shell_interpolate(raw: &str) -> Result<String, Box<dyn Error>> {
    fmt::shell_interpolate(raw, &|cmd| {
        let output = if cfg!(target_os = "windows") {
            Command::new("cmd").args(&["/C", cmd]).output()?
        } else {
            Command::new("sh").args(&["-c", cmd]).output()?
        };

        Ok(str::from_utf8(&output.stdout)?.trim_end().to_string())
    })
}

impl DockerRun {
    pub fn run(
        &self,
        docker_cmd: &Path,
        _envs: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        // Convert all options into flags
        let command_flags = self.command.as_ref().map_or(vec![], |cmds| {
            cmds.iter()
                .map(|cmd| {
                    shell_interpolate(cmd).expect("Invalid env for cmds")
                })
                .collect()
        });

        let entrypoint_flag =
            self.entrypoint.as_ref().map_or(vec![], |entrypoint| {
                vec![
                    "--entrypoint".to_string(),
                    shell_interpolate(entrypoint)
                        .expect("Invalid env for entrypoint"),
                ]
            });

        let envs_flags = self.envs.as_ref().map_or(vec![], |envs| {
            envs.iter()
                .flat_map(|(k, v)| {
                    vec![
                        "-e".to_string(),
                        shell_interpolate(&format!("{}={}", k, v))
                            .expect("Invalid env for envs"),
                    ]
                })
                .collect()
        });

        let env_file_flags =
            self.env_file.as_ref().map_or(vec![], |env_file| {
                vec![
                    "--env-file".to_string(),
                    shell_interpolate(&format!("{}", env_file.display()))
                        .expect("Invalid env for env-file"),
                ]
            });

        let network_flags = self.network.as_ref().map_or(vec![], |network| {
            vec![shell_interpolate(&format!("--network={}", network))
                .expect("Invalid env for env-file")]
        });

        let ports_flags = self.ports.as_ref().map_or(vec![], |ports| {
            ports
                .iter()
                .flat_map(|port| {
                    vec![
                        "-p".to_string(),
                        shell_interpolate(port).expect("Invalid env for ports"),
                    ]
                })
                .collect()
        });

        let volumes_flags = self.volumes.as_ref().map_or(vec![], |volumes| {
            volumes
                .iter()
                .flat_map(|volume| {
                    vec![
                        "-v".to_string(),
                        shell_interpolate(volume)
                            .expect("Invalid env for volumes"),
                    ]
                })
                .collect()
        });

        let user_flags = self.user.as_ref().map_or(vec![], |user| {
            vec![
                "-u".to_string(),
                shell_interpolate(user).expect("Invalid env for user"),
            ]
        });

        let extra_flags =
            self.extra_flags.as_ref().map_or(vec![], |extra_flags| {
                extra_flags
                    .iter()
                    .map(|extra_flag| {
                        shell_interpolate(extra_flag)
                            .expect("Invalid env for extra flags")
                    })
                    .collect()
            });

        match self.context {
            Context::Image(ref image) => {
                let image = shell_interpolate(image)?;

                let args = [
                    // Command with default flags
                    &["run".to_string()],
                    &["--rm".to_string()],
                    // Optional flags
                    &entrypoint_flag[..],
                    &envs_flags[..],
                    &env_file_flags[..],
                    &network_flags[..],
                    &ports_flags[..],
                    &volumes_flags[..],
                    &user_flags[..],
                    &extra_flags[..],
                    // Mandatory fields
                    &[image],
                    &command_flags[..],
                ]
                .concat();

                let mut child = Command::new(docker_cmd).args(args).spawn()?;
                let _code = child.wait()?;
                Ok(())
            }
            Context::Build(ref _path) => unimplemented!(),
        }
    }
}

pub fn get_cli_path() -> Result<PathBuf, which::Error> {
    which("docker")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_dockerrun(image: &str) -> DockerRun {
        DockerRunBuilder::default()
            .context(Context::Image(image.to_string()))
            .build()
            .unwrap()
    }

    #[test]
    fn test_run() {
        let mut dr = make_dockerrun("clux/muslrust:stable");
        dr.command = Some(vec!["cargo".to_string(), "--version".to_string()]);

        let docker_cmd = get_cli_path().unwrap();
        dr.run(&docker_cmd, &HashMap::new()).unwrap();
    }
}
