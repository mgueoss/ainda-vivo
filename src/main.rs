#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod atmosphere;
mod desktop;
mod diagnostics;
mod interaction;
#[cfg(feature = "dev-lab")]
mod lab;
mod preferences;
mod psychology;
mod reality;
mod scene;
mod smoke;
mod ui;
mod vhs;

fn main() -> bevy::app::AppExit {
    app::run()
}
