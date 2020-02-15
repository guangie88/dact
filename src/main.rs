mod docker;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    docker::get_docker_cli_path().unwrap();

    Ok(())
}
