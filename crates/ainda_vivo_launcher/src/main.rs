#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

#[cfg(windows)]
mod check;
#[cfg(any(windows, test))]
mod pe;
#[cfg(windows)]
mod ui;

#[cfg(windows)]
fn wide(text: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    text.as_ref().encode_wide().chain([0]).collect()
}

#[cfg(windows)]
fn main() {
    use std::{path::PathBuf, ptr::null_mut};
    use windows_sys::Win32::{
        System::LibraryLoader::{LOAD_LIBRARY_SEARCH_SYSTEM32, SetDefaultDllDirectories},
        UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW},
    };
    unsafe {
        SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32);
    }
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if let [mode, target, flag, report] = args.as_slice()
        && mode == "--inspect"
        && flag == "--report"
    {
        let status = match pe::inspect(&PathBuf::from(target)) {
            Ok(image) => report_result(
                report,
                &format!(
                    "machine=0x{:04x}\nimports={}\ndelayed={}\n",
                    image.machine,
                    image.imports.join(","),
                    image.delayed.join(",")
                ),
                0,
            ),
            Err(error) => report_result(report, &error.to_string(), 1),
        };
        std::process::exit(status);
    }
    let root = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(std::path::Path::to_path_buf));
    let package = root
        .ok_or_else(|| "Não foi possível localizar a pasta do jogo.".to_owned())
        .and_then(|root| check::Package::read(&root));
    if let [mode, flag, report] = args.as_slice()
        && mode == "--verify"
        && flag == "--report"
    {
        let status = match package {
            Ok(package) => {
                let result = check::Assessment::inspect(&package);
                let status = match result.state {
                    check::Readiness::Ready => 0,
                    check::Readiness::NeedsRuntime => 2,
                    check::Readiness::Blocked => 1,
                };
                report_result(
                    report,
                    &format!(
                        "version={}\nstate={:?}\n{}\n",
                        package.version, result.state, result.details
                    ),
                    status,
                )
            }
            Err(error) => report_result(report, &error, 1),
        };
        std::process::exit(status);
    }
    let result = package.and_then(|package| {
        let log = check::Log::open(&package.version)
            .map_err(|error| format!("Não foi possível criar o diagnóstico: {error}"))?;
        ui::run(package, log)
            .map_err(|error| format!("Não foi possível abrir o inicializador: {error}"))
    });
    if let Err(message) = result {
        unsafe {
            MessageBoxW(
                null_mut(),
                wide(&message).as_ptr(),
                wide("AINDA VIVO").as_ptr(),
                MB_ICONERROR,
            );
        }
        std::process::exit(1);
    }
}

#[cfg(windows)]
fn report_result(path: &std::ffi::OsStr, value: &str, exit_code: i32) -> i32 {
    if std::fs::write(path, value).is_ok() {
        exit_code
    } else {
        3
    }
}

#[cfg(not(windows))]
fn main() {}
