use anyhow::{Context, Result};
use std::fs;

use super::file::load_config;
use super::init::STARTER_CONFIG;

/// Handle --init-config command
pub fn handle_init_config() -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("Cannot determine config directory")?
        .join("t-rec");

    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        println!("Config file already exists: {}", config_path.display());
        return Ok(());
    }

    // Create directory if needed
    fs::create_dir_all(&config_dir).with_context(|| {
        format!(
            "Failed to create config directory: {}",
            config_dir.display()
        )
    })?;

    fs::write(&config_path, STARTER_CONFIG)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    println!("Created config file: {}", config_path.display());
    println!("Edit it to customize your t-rec settings.");

    Ok(())
}

/// Handle --list-profiles command
pub fn handle_list_profiles() -> Result<()> {
    match load_config()? {
        Some(config) => {
            if config.profiles.is_empty() {
                println!("No profiles defined in config file.");
                println!("Add profiles to your config file like this:");
                println!();
                println!("[profiles.demo]");
                println!("wallpaper = \"ventura\"");
                println!("wallpaper-padding = 100");
            } else {
                println!("Available profiles:");
                for name in config.profiles.keys() {
                    println!("  - {}", name);
                }
            }
        }
        None => {
            println!("No config file found.");
            println!("Run `t-rec --init-config` to create one.");
        }
    }
    Ok(())
}
