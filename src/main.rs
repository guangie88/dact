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

type DactConfig = HashMap<String, docker::DockerRun>;

const DACT_CONFIG_TOML_PATH: &str = "dact.toml";
const DACT_CONFIG_YAML_PATH: &str = "dact.yml";

// Exit status codes
const DACT_ACTION_MISSING: i32 = 1;
const CONFIG_MISSING: i32 = 10;
const MULTIPLE_CONFIG_FOUND: i32 = 11;

#[derive(Debug, StructOpt)]
enum DactSubOpt {
    /// Lists all possible Dact actions
    List,

    /// Runs a Dact action
    Run { action: String },
}

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Dact - Simplify running batch action within Docker container"
)]
struct DactOpt {
    #[structopt(subcommand)]
    sub: DactSubOpt,
}

fn get_config() -> Result<DactConfig, Box<dyn Error>> {
    let yaml_path = Path::new(DACT_CONFIG_YAML_PATH);
    let toml_path = Path::new(DACT_CONFIG_TOML_PATH);

    let yaml_exists = yaml_path.exists();
    let toml_exists = toml_path.exists();

    match (yaml_exists, toml_exists) {
        (true, true) => {
            eprintln!("Multiple Dact config files detected, only one config file allowed!");
            exit(MULTIPLE_CONFIG_FOUND);
        }
        (false, false) => {
            eprintln!(
                "Both {} and {} do not exist, need a config file to proceed.",
                DACT_CONFIG_YAML_PATH, DACT_CONFIG_TOML_PATH
            );

            exit(CONFIG_MISSING);
        }
        (true, _) => {
            let config_str = fs::read_to_string(&toml_path)?;
            let config: DactConfig = serde_yaml::from_str(&config_str)?;
            Ok(config)
        }
        (_, true) => {
            let config_str = fs::read_to_string(&toml_path)?;
            let config: DactConfig = toml::from_str(&config_str)?;
            Ok(config)
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = DactOpt::from_args();
    let config = get_config()?;
    let docker_cmd = docker::get_cli_path()?;

    let kv: HashMap<String, String> = env::vars().into_iter().collect();

    match opt.sub {
        DactSubOpt::List => {
            let sorted_config: BTreeMap<_, _> = config.iter().collect();

            for (action, dr) in sorted_config.iter() {
                if let Some(help) = &dr.help {
                    println!("{} - {}", action.blue().bold(), help);
                } else {
                    println!("{}", action.blue().bold());
                }
            }
        }
        DactSubOpt::Run { action } => match config.get(&action) {
            Some(dr) => dr.run(&docker_cmd, &kv)?,
            None => {
                eprintln!(
                    "Dact action [{}] does not exist!",
                    action.blue().bold()
                );
                exit(DACT_ACTION_MISSING);
            }
        },
    }

    Ok(())
}
