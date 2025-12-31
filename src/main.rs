// SPDX-FileCopyrightText: © 2025 Tenstorrent Inc.
// SPDX-License-Identifier: Apache-2.0

//! `tt-glx-eth-isolation` CLI entrypoint.
//!
//! Provides command-line operations to:
//! - Build a `(UBB -> ASIC)` mapping from detected PCIe devices
//! - Isolate/restore UBB trays by flashing ETH port-disable masks
//! - (WIP) Explore graph-based isolation via `--graph`

use std::io;

use clap::Parser;
use color_eyre::eyre;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use eyre::{Error, Result};
use luwen::pci::detect_chips;
use owo_colors::OwoColorize;
use ratatui::{Terminal, backend::CrosstermBackend};
use std::str::FromStr;

mod backend;
mod config_parsing;
mod constants;
mod graph;
mod ui;

/// Mode for config file operations.
#[derive(Debug, Clone)]
enum ConfigMode {
    /// Isolate groups of ASICs based on config file.
    Isolate,
    /// Restore groups of ASICs to default settings based on config file.
    Restore,
}

impl FromStr for ConfigMode {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "isolate" => Ok(ConfigMode::Isolate),
            "restore" => Ok(ConfigMode::Restore),
            _ => Err(eyre::eyre!(
                "Invalid mode '{}'. Must be 'isolate' or 'restore'",
                s
            )),
        }
    }
}

/// Command-line arguments for `tt-glx-eth-isolation`.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Isolate all UBB trays from each other.
    #[arg(long)]
    isolate_all_trays: bool,
    /// Isolate a specific UBB tray from others. Provide UBB number (0-indexed).
    #[arg(long)]
    isolate_tray: Option<usize>,
    /// Restore all ASICs to default Ethernet settings.
    #[arg(long)]
    restore_all: bool,
    /// Restore a specific UBB tray to default Ethernet settings. Provide UBB number (0-indexed).
    #[arg(long)]
    restore_tray: Option<usize>,
    /// Launch the GUI interface. (Does not work currently - just a template for now)
    #[arg(long)]
    gui: bool,
    /// Use a TOML config file to specify groups of ASICs. Mode can be 'isolate' or 'restore'.
    /// Usage: --use-config <isolate|restore> <filename>
    ///   - isolate: Isolate groups of ASICs based on the config file
    ///   - restore: Restore groups of ASICs to default settings based on the config file
    #[arg(long, num_args = 2, value_names = ["MODE", "FILENAME"])]
    use_config: Option<Vec<String>>,
    /// Generate a template TOML config file. Optionally specify a filename, otherwise uses ~/.config/tenstorrent/<date>_<time>_glx_eth_config.toml
    #[arg(long, num_args = 0..=1)]
    gen_config: Option<Option<String>>,
}

/// Program entrypoint.
fn main() -> Result<()> {
    // Install color_eyre for better error reporting
    color_eyre::install()?;
    let args = Args::parse();

    // Handle config generation early, before device detection
    if let Some(filename_opt) = args.gen_config {
        let result = config_parsing::generate_config_file(filename_opt);
        if let Err(e) = result {
            eprintln!("Failed to generate config file: {}", e);
            return Err(e);
        }
        println!("Config file generated successfully");
        return Ok(());
    }

    println!("{}", "WARNING: This script should not be run in a VM or Docker as the PCIe bus IDs may be different...".yellow());
    let chips = detect_chips()
        .expect("Failed to detect PCI devices, can you see the accelerators on lspci?");

    // Cannot proceed if not exactly 32 chips are detected
    if chips.len() != 32 {
        Err(eyre::eyre!(
            "Didn't detect 32 chips, detected {} instead. - are you running in a container or VM? Maybe a UBB tray needs reseating. Exiting...",
            chips.len()
        ))?;
    }

    let ubb_device_map = backend::build_ubb_device_map(chips)?;
    let graph = graph::generate_galaxy_graph();
    let cached_graph = graph::CachedGraph::new(graph);

    if let Some(config_args) = args.use_config {
        if config_args.len() != 2 {
            return Err(eyre::eyre!(
                "Expected 2 arguments for --use-config: <mode> <filename>"
            ));
        }
        let mode_str = &config_args[0];
        let config_path = &config_args[1];
        let mode = ConfigMode::from_str(mode_str)?;
        let config = config_parsing::load_config_from_file(config_path)?;
        let graph_group_coords = config.groups_to_graph_coords()?;

        match mode {
            ConfigMode::Isolate => {
                for group in graph_group_coords.iter() {
                    if !cached_graph.is_subset_connected_union_find(group) {
                        println!("Group is not connected");
                        return Ok(());
                    }
                    let edges_to_disable = cached_graph.get_edges_to_disable(group);
                    backend::flash_edges_to_disable(edges_to_disable, &ubb_device_map)?;
                }
            }
            ConfigMode::Restore => {
                for group in graph_group_coords.iter() {
                    backend::reset_group_of_asics(group.clone(), &ubb_device_map)?;
                }
                println!("{}", "Flashed default ETH port disables. Please run `tt-smi -glx_reset` to apply changes.".yellow());
            }
        }
        return Ok(());
    }
    // Isolation Options
    if args.isolate_all_trays {
        for ubb_num in 1..ubb_device_map.len() + 1 {
            backend::isolate_ubb(ubb_num, &ubb_device_map)?;
        }
        println!("{}", "Please run `tt-smi -glx_reset` to apply changes.".yellow());
    }
    if let Some(ubb_num) = args.isolate_tray {
        if ubb_num > ubb_device_map.len() || ubb_num < 1 {
            println!("{}", format!("Invalid UBB number: {}. Valid range: 1-{}", ubb_num, ubb_device_map.len()).red());
            return Ok(());
        }
        backend::isolate_ubb(ubb_num, &ubb_device_map)?;
        println!("{}", "Please run `tt-smi -glx_reset` to apply changes.".yellow());
    }
    // Restoration Options
    if args.restore_all {
        for ubb_num in 1..ubb_device_map.len() + 1 {
            backend::set_default_ubb_eth_disables(ubb_num, &ubb_device_map)?;
        }
        println!("{}", "Please run `tt-smi -glx_reset` to apply changes.".yellow());
    }
    if let Some(ubb_num) = args.restore_tray {
        if ubb_num > ubb_device_map.len() || ubb_num < 1 {
            println!("{}", format!("Invalid UBB number: {}. Valid range: 1-{}", ubb_num, ubb_device_map.len()).red());
            return Ok(());
        }
        backend::set_default_ubb_eth_disables(ubb_num, &ubb_device_map)?;
        println!("{}", "Please run `tt-smi -glx_reset` to apply changes.".yellow());
    }
    // GUI is work in progress hence not true by default.
    // Currently just a template to build upon.
    if args.gui {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create app and run
        let mut app = ui::App::new();
        let res = ui::run_app(&mut terminal, &mut app);

        // Restore terminal
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{:?}", err);
        }
    }
    Ok(())
}
