use std::io;

use clap::Parser;
use color_eyre::eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use luwen::pci::detect_chips;
use ratatui::{Terminal, backend::CrosstermBackend};

mod backend;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Isolate all UBB trays from each other.
    #[arg(long)]
    isolate_all_trays: bool,
    /// Isolate a specific UBB tray from others. Provide UBB number (0-indexed).
    #[arg(long)]
    isolate_tray: Option<usize>,
    /// Restore all UBB trays to default Ethernet settings.
    #[arg(long)]
    restore_all_trays: bool,
    /// Restore a specific UBB tray to default Ethernet settings. Provide UBB number (0-indexed).
    #[arg(long)]
    restore_tray: Option<usize>,
    /// Launch the GUI interface. (Does not work currently - just a template for now)
    #[arg(long)]
    gui: bool,
}

fn main() -> Result<()> {
    // Install color_eyre for better error reporting
    color_eyre::install()?;
    let args = Args::parse();
    println!(
        "{}WARNING: This script should not be run in a VM or Docker as the PCIe bus IDs may be different...{}",
        backend::CmdLineColor::YELLOW,
        backend::CmdLineColor::ENDC
    );
    let chips = detect_chips()
        .expect("Failed to detect PCI devices, can you see the accelerators on lspci?");

    // Cannot proceed if not exactly 32 chips are detected
    if chips.len() != 32 {
        Err(color_eyre::eyre::eyre!(
            "Didn't detect 32 chips, detected {} instead. - are you running in a container or VM? Maybe a UBB tray needs reseating. Exiting...",
            chips.len()
        ))?;
    }
    let ubb_device_map = backend::get_ubb_device_map(chips)?;
    // Isolation Options
    if args.isolate_all_trays {
        for ubb_num in 1..ubb_device_map.len() + 1 {
            backend::isolate_ubb(ubb_num, &ubb_device_map)?;
        }
        println!(
            "{}Please run `tt-smi -glx_reset` to apply changes.{}",
            backend::CmdLineColor::YELLOW,
            backend::CmdLineColor::ENDC
        );
    }
    if let Some(ubb_num) = args.isolate_tray {
        if ubb_num > ubb_device_map.len() || ubb_num < 1 {
            println!(
                "{}Invalid UBB number: {}. Valid range: 1-{}{}",
                backend::CmdLineColor::RED,
                ubb_num,
                ubb_device_map.len(),
                backend::CmdLineColor::ENDC
            );
            return Ok(());
        }
        backend::isolate_ubb(ubb_num, &ubb_device_map)?;
        println!(
            "{}Please run `tt-smi -glx_reset` to apply changes.{}",
            backend::CmdLineColor::YELLOW,
            backend::CmdLineColor::ENDC
        );
    }
    // Restoration Options
    if args.restore_all_trays {
        for ubb_num in 1..ubb_device_map.len() + 1 {
            backend::set_default_ubb_eth_disables(ubb_num, &ubb_device_map)?;
        }
        println!(
            "{}Please run `tt-smi -glx_reset` to apply changes.{}",
            backend::CmdLineColor::YELLOW,
            backend::CmdLineColor::ENDC
        );
    }
    if let Some(ubb_num) = args.restore_tray {
        if ubb_num > ubb_device_map.len() || ubb_num < 1 {
            println!(
                "{}Invalid UBB number: {}. Valid range: 1-{}{}",
                backend::CmdLineColor::RED,
                ubb_num,
                ubb_device_map.len(),
                backend::CmdLineColor::ENDC
            );
            return Ok(());
        }
        backend::set_default_ubb_eth_disables(ubb_num, &ubb_device_map)?;
        println!(
            "{}Please run `tt-smi -glx_reset` to apply changes.{}",
            backend::CmdLineColor::YELLOW,
            backend::CmdLineColor::ENDC
        );
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
