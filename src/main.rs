use std::io;

use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use luwen::pci::detect_chips;
use ratatui::{backend::CrosstermBackend, Terminal};

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


fn main() -> io::Result<()> {
    let args = Args::parse();
    match detect_chips() {
        Ok(chips) => {
            println!("{}Detected {} chip(s){}",backend::CmdLineColor::PURPLE, chips.len(), backend::CmdLineColor::ENDC);
            let ubb_device_map = backend::get_ubb_device_map(chips).unwrap();
            // Isolation Options
            if args.isolate_all_trays {
                for ubb_num in 1..ubb_device_map.len()+1 {
                    backend::isolate_ubb(ubb_num, &ubb_device_map).unwrap();
                }
                println!("{}Please run `tt-smi -glx_reset` to apply changes.{}", backend::CmdLineColor::YELLOW, backend::CmdLineColor::ENDC);
            }
            if let Some(ubb_num) = args.isolate_tray {
                if ubb_num > ubb_device_map.len() || ubb_num < 1 {
                    println!("{}Invalid UBB number: {}. Valid range: 1-{}{}",
                            backend::CmdLineColor::RED, ubb_num, ubb_device_map.len(), backend::CmdLineColor::ENDC);
                    return Ok(());
                }
                backend::isolate_ubb(ubb_num, &ubb_device_map).unwrap();
                println!("{}Please run `tt-smi -glx_reset` to apply changes.{}", backend::CmdLineColor::YELLOW, backend::CmdLineColor::ENDC);
            }
            // Restoration Options
            if args.restore_all_trays {
                for ubb_num in 1..ubb_device_map.len()+1 {
                    backend::set_default_ubb_eth_disables(ubb_num, &ubb_device_map).unwrap();
                }
                println!("{}Please run `tt-smi -glx_reset` to apply changes.{}", backend::CmdLineColor::YELLOW, backend::CmdLineColor::ENDC);
            }
            if let Some(ubb_num) = args.restore_tray {
                if ubb_num > ubb_device_map.len() || ubb_num < 1 {
                    println!("{}Invalid UBB number: {}. Valid range: 1-{}{}",
                            backend::CmdLineColor::RED, ubb_num, ubb_device_map.len(), backend::CmdLineColor::ENDC);
                    return Ok(());
                }
                backend::set_default_ubb_eth_disables(ubb_num, &ubb_device_map).unwrap();
                println!("{}Please run `tt-smi -glx_reset` to apply changes.{}", backend::CmdLineColor::YELLOW, backend::CmdLineColor::ENDC);
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
            return Ok(());
        }
        Err(e) => {
            println!("Error detecting chips: {:?}", e);
            return Ok(());
        }
    }
}
