// SPDX-FileCopyrightText: © 2025 Tenstorrent Inc.
// SPDX-License-Identifier: Apache-2.0

//! TOML config schema (deserialization).
//!
//! This module defines the structures used to deserialize a TOML config file via `serde`.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use color_eyre::eyre::{Result, WrapErr, eyre};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::graph::GraphCoords;

/// Load a TOML config from `path` and deserialize it into [`Config`].
pub fn load_config_from_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path = path.as_ref();
    let contents = std::fs::read_to_string(path)
        .wrap_err_with(|| format!("Failed to read config file '{}'", path.display()))?;
    let config: Config = toml::from_str(&contents).wrap_err(
        "Failed to parse TOML config into Config. Please check the format and try again.",
    )?;
    Ok(config)
}

/// Top-level configuration document.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// System identifier
    pub system: String,
    /// Date string from the config.
    pub date: String,
    /// Time string from the config.
    pub time: String,
    /// Group definitions in the config.
    pub groups: Vec<Group>,
}

impl Config {
    /// Convert each group's devices into [`GraphCoords`] using `(ubb_num - 1, asic_num - 1)`.
    ///
    /// The returned outer vector matches the order of `self.groups` (i.e. group index is the
    /// vector index).
    pub fn groups_to_graph_coords(&self) -> Result<Vec<Vec<GraphCoords>>> {
        self.groups
            .iter()
            .map(|g| g.devices_to_graph_coords())
            .collect()
    }
}

/// A user-defined group of devices.
#[derive(Debug, Deserialize, Serialize)]
pub struct Group {
    /// Group id / label.
    pub group: u32,
    /// Devices belonging to this group.
    pub devices: Vec<Device>,
}

impl Group {
    /// Convert this group's devices into [`GraphCoords`] using `(ubb_num - 1, asic_num - 1)`.
    pub fn devices_to_graph_coords(&self) -> Result<Vec<GraphCoords>> {
        self.devices.iter().map(Device::to_graph_coords).collect()
    }
}

/// A single device entry in the config (UBB + ASIC coordinates).
#[derive(Debug, Deserialize, Serialize)]
pub struct Device {
    /// UBB index.
    pub ubb_num: u32,
    /// ASIC index within the UBB.
    pub asic_num: u32,
}

impl Device {
    /// Convert this device entry into [`GraphCoords`] using `(ubb_num - 1, asic_num - 1)`.
    pub fn to_graph_coords(&self) -> Result<GraphCoords> {
        if self.ubb_num < 1 {
            return Err(eyre!("ubb_num must be >= 1"));
        }
        if self.asic_num < 1 {
            return Err(eyre!("asic_num must be >= 1"));
        }
        Ok(GraphCoords(
            (self.ubb_num - 1) as usize,
            (self.asic_num - 1) as usize,
        ))
    }
}

pub fn generate_config_file(filename: Option<impl AsRef<Path>>) -> Result<()> {
    let system = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let time = chrono::Local::now().format("%H:%M:%S").to_string();
    let config = Config {
        system: system.clone(),
        date: date.clone(),
        time: time.clone(),
        groups: vec![
            Group {
                group: 1,
                devices: vec![],
            },
            Group {
                group: 2,
                devices: vec![],
            },
        ],
    };

    let config_file = match filename {
        Some(path) => PathBuf::from(path.as_ref()),
        None => {
            // Use XDG config directory: ~/.config/tenstorrent/ or fallback to current directory
            let dir = if let Some(home) = std::env::var_os("HOME") {
                println!("Using home directory for config file: {:?}", home);
                let mut config_dir = PathBuf::from(home);
                config_dir.push(".config");
                config_dir.push("tenstorrent");
                config_dir
            } else {
                println!("Using current directory for config file");
                PathBuf::from(".config")
            };
            std::fs::create_dir_all(&dir)
                .wrap_err_with(|| format!("Failed to create directory '{}'", dir.display()))?;
            dir.join(format!("{}_{}_glx_eth_config.toml", date, time))
        }
    };

    std::fs::write(&config_file, toml::to_string(&config)?)
        .wrap_err_with(|| format!("Failed to write config file '{}'", config_file.display()))?;
    println!("{}", format!("Successfully created file: {}", config_file.display()).green());
    Ok(())
}
