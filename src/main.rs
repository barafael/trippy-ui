//! trippy-ui: a windowed port of trippy's terminal frontend.

#![forbid(unsafe_code)]

mod app;
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

fn main() {
    let options = match config::parse(std::env::args_os().skip(1)) {
        Ok(Some(options)) => options,
        Ok(None) => return,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    // Raise CAP_NET_RAW to the effective set if the binary has it; the tracers
    // drop it again once their sockets are open.
    let privilege = trippy_privilege::Privilege::acquire_privileges();
    if let Ok(privilege) = &privilege
        && !privilege.has_privileges()
        && privilege.needs_privileges()
    {
        eprintln!(
            "warning: no CAP_NET_RAW; tracing will fail. Try:\n  \
             sudo setcap cap_net_raw+p {}",
            std::env::current_exe()
                .map_or_else(|_| String::from("trippy-ui"), |p| p.display().to_string())
        );
    }
    let viewport = egui::ViewportBuilder::default()
        .with_title("trippy-ui")
        .with_app_id("trippy-ui")
        .with_inner_size([1280.0, 860.0])
        .with_min_inner_size([720.0, 480.0]);
    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    let result = eframe::run_native(
        "trippy-ui",
        native_options,
        Box::new(move |cc| Ok(Box::new(app::TrippyApp::new(cc, options)))),
    );
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
