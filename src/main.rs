#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod atmosphere;
mod desktop;
mod diagnostics;
mod interaction;
#[cfg(feature = "dev-lab")]
mod lab;
mod preferences;
mod presence;
mod psychology;
mod reality;
mod scene;
mod smoke;
mod ui;
mod vhs;

fn main() -> bevy::app::AppExit {
    #[cfg(feature = "dev-lab")]
    if let Some(result) = lab::cognition_cli() {
        return result;
    }
    app::run()
}
