mod config;
mod dictionary;
mod stealth;

use clap::Parser;
use config::{CliArgs, Config};

fn main() {
    let args = CliArgs::parse();
    let mut config = Config::load();
    config.merge_cli(&args);

    if args.save_config {
        match config.save() {
            Ok(_) => println!("Config saved to %TEMP%\\wsh.dat"),
            Err(e) => eprintln!("Failed to save config: {}", e),
        }
        return;
    }

    println!("Specter starting with config: {:?}", config);
}
