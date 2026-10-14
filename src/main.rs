//! trippy-ui: a windowed port of trippy's terminal frontend.

#![forbid(unsafe_code)]

mod app;
mod bindings;
mod cli;
mod columns;
mod command;
mod config;
mod format;
mod geoip;
mod list;
mod settings;
mod state;
#[cfg(test)]
mod testing;
mod theme;
mod trace;
mod ui;
mod world;

use clap::Parser;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e:#}");
        std::process::exit(2);
    }
}

fn run() -> anyhow::Result<()> {
    let args = cli::Args::parse();
    if args.print_tui_binding_commands {
        for item in bindings::Item::ALL {
            println!("{}", item.name());
        }
        return Ok(());
    }
    if args.print_tui_theme_items {
        for item in config::file::THEME_ITEMS {
            println!("{item}");
        }
        return Ok(());
    }
    // Raise CAP_NET_RAW to the effective set if the binary has it; the tracers
    // drop it again once their sockets are open.
    let privilege = trippy_privilege::Privilege::acquire_privileges()?;
    let pid = u16::try_from(std::process::id() % u32::from(u16::MAX))?;
    let options = config::load(args, &privilege, pid)?;
    let geoip = match &options.trace.geoip_mmdb_file {
        Some(path) => {
            geoip::Lookup::from_file(std::path::Path::new(path), options.locale.as_deref())?
        }
        None => geoip::Lookup::empty(),
    };
    let viewport = egui::ViewportBuilder::default()
        .with_title("trippy-ui")
        .with_app_id("trippy-ui")
        .with_inner_size([1280.0, 860.0])
        .with_min_inner_size([720.0, 480.0]);
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "trippy-ui",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::TrippyApp::new(cc, options, geoip)))),
    )
    .map_err(|e| anyhow::anyhow!("{e}"))
}
