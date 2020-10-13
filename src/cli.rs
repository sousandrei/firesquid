use clap::{load_yaml, App};

use crate::error;

pub struct CliOptions {
    pub port: u16,
    pub tmp_dir: String,
    pub log_dir: String,
    pub assets_dir: String,
    pub drive_name: String,
    pub kernel_name: String,
}

pub fn generate_cli() -> Result<CliOptions, Box<dyn std::error::Error + Send + Sync>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let tmp_dir = matches
        .value_of("tmp_dir")
        .ok_or(error::RuntimeError::new("Invalid parameter [tmp_dir]"))?;

    let log_dir = matches
        .value_of("log_dir")
        .ok_or(error::RuntimeError::new("Invalid parameter [log_dir]"))?;

    let assets_dir = matches
        .value_of("assets_dir")
        .ok_or(error::RuntimeError::new("Invalid parameter [assets_dir]"))?;

    let drive_name = matches
        .value_of("drive_name")
        .ok_or(error::RuntimeError::new("Invalid parameter [drive_name]"))?;

    let kernel_name = matches
        .value_of("kernel_name")
        .ok_or(error::RuntimeError::new("Invalid parameter [kernel_name]"))?;

    let port = matches
        .value_of("port")
        .ok_or(error::RuntimeError::new("Invalid parameter [port]"))?;

    let port: u16 = port.parse()?;

    let cli_options = CliOptions {
        port: port,
        tmp_dir: String::from(tmp_dir),
        log_dir: String::from(log_dir),
        assets_dir: String::from(assets_dir),
        drive_name: String::from(drive_name),
        kernel_name: String::from(kernel_name),
    };

    Ok(cli_options)
}
