// #![deny(warnings)]
mod docker;

use colored::*;
use serde_yaml;
use std::collections::{BTreeMap, HashMap};
use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::exit;
use structopt::StructOpt;
use toml;

type DoaConfig = HashMap<String, docker::DockerRun>;

const DRUN_CONFIG_TOML_PATH: &str = "doa.toml";
const DRUN_CONFIG_YAML_PATH: &str = "doa.yml";

// Exit status codes
const DRUN_ACTION_MISSING: i32 = 1;
const CONFIG_MISSING: i32 = 10;
const MULTIPLE_CONFIG_FOUND: i32 = 11;

#[derive(Debug, StructOpt)]
enum DoaSubOpt {
    /// Lists all possible doa actions
    List,

    /// Runs a doa action
    Run { action: String },
}

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Doa - Simplify running batch action within Docker container"
)]
struct DoaOpt {
    #[structopt(subcommand)]
    sub: DoaSubOpt,
}

fn get_config() -> Result<DoaConfig, Box<dyn Error>> {
    let yaml_path = Path::new(DRUN_CONFIG_YAML_PATH);
    let toml_path = Path::new(DRUN_CONFIG_TOML_PATH);

    let yaml_exists = yaml_path.exists();
    let toml_exists = toml_path.exists();

    match (yaml_exists, toml_exists) {
        (true, true) => {
            eprintln!("Multiple Doa config files detected, only one config file allowed!");
            exit(MULTIPLE_CONFIG_FOUND);
        }
        (false, false) => {
            eprintln!(
                "Both {} and {} do not exist, need a config file to proceed.",
                DRUN_CONFIG_YAML_PATH, DRUN_CONFIG_TOML_PATH
            );

            exit(CONFIG_MISSING);
        }
        (true, _) => {
            let config_str = fs::read_to_string(&toml_path)?;
            let config: DoaConfig = serde_yaml::from_str(&config_str)?;
            Ok(config)
        }
        (_, true) => {
            let config_str = fs::read_to_string(&toml_path)?;
            let config: DoaConfig = toml::from_str(&config_str)?;
            Ok(config)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = DoaOpt::from_args();
    let config = get_config()?;
    let docker_cmd = docker::get_cli_path()?;

    let kv: HashMap<String, String> = env::vars().into_iter().collect();

    match opt.sub {
        DoaSubOpt::List => {
            let sorted_config: BTreeMap<_, _> = config.iter().collect();

            for (action, dr) in sorted_config.iter() {
                if let Some(help) = &dr.help {
                    println!("{} - {}", action.blue().bold(), help);
                } else {
                    println!("{}", action.blue().bold());
                }
            }
        }
        DoaSubOpt::Run { action } => match config.get(&action) {
            Some(dr) => dr.run(&docker_cmd, &kv)?,
            None => {
                eprintln!(
                    "Doa action [{}] does not exist!",
                    action.blue().bold()
                );
                exit(DRUN_ACTION_MISSING);
            }
        },
    }

    Ok(())
}
