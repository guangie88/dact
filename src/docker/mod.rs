use mustache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;
use which::which;

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerRun {
    pub image: String,
    pub help: Option<String>,

    pub interactive: Option<bool>,
    pub tty: Option<bool>,

    pub command: Option<Vec<String>>,
    pub entrypoint: Option<String>,
    pub envs: Option<HashMap<String, String>>,
    pub env_file: Option<PathBuf>,
    pub volumes: Option<Vec<String>>,
    pub user: Option<String>,
    pub extra_flags: Option<Vec<String>>,
}

pub fn interpolate_host_envs(
    raw: &str,
    kv: &HashMap<String, String>,
) -> Result<String, Box<dyn Error>> {
    let tmpl = mustache::compile_str(raw)?;
    let mut buf = Vec::new();
    tmpl.render(&mut buf, &kv)?;

    let rendered_str = str::from_utf8(&buf)?;
    Ok(rendered_str.to_string())
}

impl DockerRun {
    pub fn run(
        &self,
        docker_cmd: &Path,
        kv: &HashMap<String, String>,
    ) -> Result<(), Box<dyn Error>> {
        // Convert all options into flags
        let command_flags = self.command.as_ref().map_or(vec![], |cmds| {
            cmds.iter()
                .map(|cmd| {
                    interpolate_host_envs(cmd, kv)
                        .expect("Invalid env for cmds")
                })
                .collect()
        });

        let entrypoint_flag =
            self.entrypoint.as_ref().map_or(vec![], |entrypoint| {
                vec![
                    "--entrypoint".to_string(),
                    interpolate_host_envs(entrypoint, kv)
                        .expect("Invalid env for entrypoint"),
                ]
            });

        let envs_flags = self.envs.as_ref().map_or(vec![], |envs| {
            envs.iter()
                .flat_map(|(k, v)| {
                    vec![
                        "-e".to_string(),
                        interpolate_host_envs(&format!("{}={}", k, v), kv)
                            .expect("Invalid env for envs"),
                    ]
                })
                .collect()
        });

        let env_file_flags =
            self.env_file.as_ref().map_or(vec![], |env_file| {
                vec![
                    "--env-file".to_string(),
                    interpolate_host_envs(
                        &format!("{}", env_file.display()),
                        kv,
                    )
                    .expect("Invalid env for env-file"),
                ]
            });

        let volumes_flags = self.volumes.as_ref().map_or(vec![], |volumes| {
            volumes
                .iter()
                .flat_map(|volume| {
                    vec![
                        "-v".to_string(),
                        interpolate_host_envs(volume, kv)
                            .expect("Invalid env for volumes"),
                    ]
                })
                .collect()
        });

        let user_flags = self.user.as_ref().map_or(vec![], |user| {
            vec![
                "-u".to_string(),
                interpolate_host_envs(user, kv).expect("Invalid env for user"),
            ]
        });

        let extra_flags =
            self.extra_flags.as_ref().map_or(vec![], |extra_flags| {
                extra_flags
                    .iter()
                    .map(|extra_flag| {
                        interpolate_host_envs(extra_flag, kv)
                            .expect("Invalid env for extra flags")
                    })
                    .collect()
            });

        let image = interpolate_host_envs(&self.image, kv)?;

        let args = [
            // Command with default flags
            &["run".to_string()],
            &["--rm".to_string()],
            // Optional flags
            &entrypoint_flag[..],
            &envs_flags[..],
            &env_file_flags[..],
            &volumes_flags[..],
            &user_flags[..],
            &extra_flags[..],
            // Mandatory fields
            &[image],
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
            interactive: None,
            tty: None,
            command: None,
            entrypoint: None,
            envs: None,
            env_file: None,
            volumes: None,
            user: None,
            extra_flags: None,
        }
    }

    #[test]
    fn test_run() {
        let mut dr = make_dockerrun("clux/muslrust:stable");
        dr.command = Some(vec!["cargo".to_string(), "--version".to_string()]);

        let docker_cmd = get_cli_path().unwrap();
        dr.run(&docker_cmd, &HashMap::new()).unwrap();
    }
}
