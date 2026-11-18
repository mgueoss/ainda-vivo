use crate::{pe, wide};
use ash::vk;
use std::{
    ffi::{OsStr, c_void},
    fs::{self, File, OpenOptions},
    io::{self, Write},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Child, Command},
    ptr::null_mut,
    time::{SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::{FreeLibrary, HMODULE},
    System::{
        LibraryLoader::{GetProcAddress, LOAD_LIBRARY_SEARCH_SYSTEM32, LoadLibraryExW},
        SystemInformation::{
            GetNativeSystemInfo, GetSystemDirectoryW, OSVERSIONINFOW, SYSTEM_INFO,
        },
    },
};

const GAME_FILE: &str = "ainda-vivo.exe";
const NO_CONSOLE: u32 = 0x0800_0000;
const REDIST_INSTALLER: &str = "vc_redist.x64.exe";

#[derive(Clone)]
pub struct Package {
    pub version: String,
    pub binary: PathBuf,
    pub folder: PathBuf,
    installer: PathBuf,
}

impl Package {
    pub fn read(root: &Path) -> Result<Self, String> {
        let folder = root.join("game");
        let manifest = fs::read_to_string(folder.join("version.txt")).map_err(|_| {
            "Extraia todo o ZIP. O arquivo game/version.txt está ausente.".to_owned()
        })?;
        let version = manifest.trim();
        if version.len() > 24
            || version.split('.').count() != 3
            || version
                .split('.')
                .any(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err("O arquivo game/version.txt contém uma versão inválida.".to_owned());
        }
        Ok(Self {
            version: version.to_owned(),
            binary: folder.join(GAME_FILE),
            folder,
            installer: root.join("dependencies").join(REDIST_INSTALLER),
        })
    }

    pub fn installer_is_local(&self) -> bool {
        self.installer.is_file()
    }
}

#[derive(Clone)]
pub struct Log {
    pub path: PathBuf,
    game: PathBuf,
}

impl Log {
    pub fn open(version: &str) -> io::Result<Self> {
        let root = std::env::var_os("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let root = if let Some(qa) = std::env::var_os("AINDA_VIVO_LAUNCHER_DATA_DIR") {
            PathBuf::from(qa)
        } else {
            root.join("AindaVivo").join("launcher")
        };
        let folder = root.join(version);
        fs::create_dir_all(&folder)?;
        let path = folder.join("inicio.log");
        OpenOptions::new().append(true).create(true).open(&path)?;
        Ok(Self {
            path,
            game: folder.join("jogo.log"),
        })
    }

    pub fn write(&self, message: &str) {
        if let Ok(mut file) = OpenOptions::new().append(true).open(&self.path) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = writeln!(file, "[{now}] {message}");
        }
    }

    pub fn launch(&self, package: &Package) -> io::Result<Child> {
        let file = File::create(&self.game)?;
        let child = Command::new(&package.binary)
            .current_dir(&package.folder)
            .stdout(file.try_clone()?)
            .stderr(file)
            .spawn()?;
        self.write(&format!(
            "Jogo iniciado. Versão {} PID {}. Saída em {}.",
            package.version,
            child.id(),
            self.game.display()
        ));
        Ok(child)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Readiness {
    Ready,
    NeedsRuntime,
    Blocked,
}

#[derive(Debug)]
pub struct Assessment {
    pub state: Readiness,
    pub status: String,
    pub dependencies: String,
    pub compatibility: String,
    pub details: String,
}

impl Assessment {
    pub fn inspect(package: &Package) -> Self {
        let image =
            match pe::inspect(&package.binary) {
                Ok(image) => image,
                Err(error) => return Self {
                    state: Readiness::Blocked,
                    status:
                        "O jogo está ausente ou o executável é inválido. Extraia o ZIP novamente."
                            .to_owned(),
                    dependencies: "Executável indisponível".to_owned(),
                    compatibility: "Verificação interrompida".to_owned(),
                    details: format!("PE {}: {error}", package.binary.display()),
                },
            };
        let os = windows();
        let graphics = os.is_some().then(gpu);
        let compatible = graphics
            .as_ref()
            .is_some_and(|gpu| gpu.dx12 || gpu.vulkan.is_some());
        let available = |name: &str| {
            system_dll(name).is_some() || pe::inspect(&package.folder.join(name)).is_ok()
        };
        let missing = missing_dlls(&image.imports, available);
        let optional = missing_dlls(&image.delayed, available);
        let state = readiness(compatible, &missing);
        let status = match (state, os, compatible) {
            (Readiness::Ready, _, _) => "Preparação concluída. Clique em JOGAR.",
            (_, None, _) => "Este pacote precisa de Windows 10 ou 11 em um PC x64.",
            (_, _, false) => {
                "Nenhuma GPU compatível foi detectada. Atualize o driver de vídeo oficial e tente novamente."
            }
            (Readiness::NeedsRuntime, _, _) => {
                "Falta o Microsoft Visual C++ x64. Clique em JOGAR para autorizar a instalação."
            }
            _ => "Faltam arquivos do jogo ou do Windows. Consulte o diagnóstico.",
        };
        let dependencies = if missing.is_empty() {
            format!(
                "{} bibliotecas de inicialização disponíveis",
                image.imports.len()
            )
        } else {
            format!("Ausentes: {}", missing.join(", "))
        };
        let compatibility = match (os, graphics) {
            (Some(build), Some(graphics)) if compatible => {
                let device = match (graphics.vulkan, graphics.dx12) {
                    (Some(name), true) => format!("Vulkan / Direct3D 12 · {name}"),
                    (Some(name), false) => format!("Vulkan · {name}"),
                    _ => "Direct3D 12".to_owned(),
                };
                format!("Windows x64 (build {build}) · {device}")
            }
            (Some(_), _) => "Windows x64 · GPU compatível não detectada".to_owned(),
            _ => "Sistema operacional incompatível".to_owned(),
        };
        let details = format!(
            "PE machine=0x{:04x}\nImports: {}\nDelay-load: {}\nMissing: {}\nDelay-load missing: {}\nCompatibility: {}",
            image.machine,
            image.imports.join(", "),
            image.delayed.join(", "),
            missing.join(", "),
            optional.join(", "),
            compatibility,
        );
        Self {
            state,
            status: status.to_owned(),
            dependencies,
            compatibility,
            details,
        }
    }
}

fn missing_dlls(names: &[String], mut available: impl FnMut(&str) -> bool) -> Vec<&str> {
    names
        .iter()
        .map(String::as_str)
        .filter(|name| !available(name))
        .collect()
}

fn readiness(compatible: bool, missing: &[&str]) -> Readiness {
    if !compatible {
        return Readiness::Blocked;
    }
    if missing.is_empty() {
        return Readiness::Ready;
    }
    if missing.iter().all(|dll| {
        matches!(
            *dll,
            "vcruntime140.dll"
                | "vcruntime140_1.dll"
                | "msvcp140.dll"
                | "msvcp140_1.dll"
                | "msvcp140_2.dll"
                | "concrt140.dll"
                | "vcomp140.dll"
        )
    }) {
        return Readiness::NeedsRuntime;
    }
    Readiness::Blocked
}

struct Library(HMODULE);

impl Drop for Library {
    fn drop(&mut self) {
        unsafe {
            FreeLibrary(self.0);
        }
    }
}

fn system_dll(name: &str) -> Option<Library> {
    let name = wide(name);
    let module = unsafe { LoadLibraryExW(name.as_ptr(), null_mut(), LOAD_LIBRARY_SEARCH_SYSTEM32) };
    (!module.is_null()).then(|| Library(module))
}

fn windows() -> Option<u32> {
    type Version = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;
    let mut system = SYSTEM_INFO::default();
    unsafe { GetNativeSystemInfo(&mut system) };
    if unsafe { system.Anonymous.Anonymous.wProcessorArchitecture } != 9 {
        return None;
    }
    let library = system_dll("ntdll.dll")?;
    let rtl = unsafe { GetProcAddress(library.0, c"RtlGetVersion".as_ptr().cast())? };
    let rtl: Version = unsafe { std::mem::transmute(rtl) };
    let mut info = OSVERSIONINFOW {
        dwOSVersionInfoSize: size_of::<OSVERSIONINFOW>() as u32,
        ..Default::default()
    };
    if unsafe { rtl(&mut info) } == 0 && info.dwMajorVersion == 10 && info.dwBuildNumber >= 14393 {
        Some(info.dwBuildNumber)
    } else {
        None
    }
}

struct Gpu {
    dx12: bool,
    vulkan: Option<String>,
}

fn gpu() -> Gpu {
    Gpu {
        dx12: probe_dx12(),
        vulkan: probe_vulkan(),
    }
}

fn probe_dx12() -> bool {
    type CreateDevice = unsafe extern "system" fn(
        *mut c_void,
        u32,
        *const windows_sys::core::GUID,
        *mut *mut c_void,
    ) -> i32;
    // Queried without creating a device. The null adapter selects the default hardware adapter.
    const IID: windows_sys::core::GUID =
        windows_sys::core::GUID::from_u128(0x189819f1_1db6_4b57_be54_1821339b85f7);
    let Some(lib) = system_dll("d3d12.dll") else {
        return false;
    };
    let Some(symbol) = (unsafe { GetProcAddress(lib.0, c"D3D12CreateDevice".as_ptr().cast()) })
    else {
        return false;
    };
    let create: CreateDevice = unsafe { std::mem::transmute(symbol) };
    (unsafe { create(null_mut(), 0xb000, &IID, null_mut()) }) >= 0
}

fn probe_vulkan() -> Option<String> {
    let loader = system_dll("vulkan-1.dll")?;
    let get_proc = unsafe { GetProcAddress(loader.0, c"vkGetInstanceProcAddr".as_ptr().cast())? };
    let get_proc: vk::PFN_vkGetInstanceProcAddr = unsafe { std::mem::transmute(get_proc) };
    let entry = unsafe {
        ash::Entry::from_static_fn(ash::StaticFn {
            get_instance_proc_addr: get_proc,
        })
    };
    let app = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_1);
    let info = vk::InstanceCreateInfo::default().application_info(&app);
    let instance = unsafe { entry.create_instance(&info, None).ok()? };
    let found = unsafe { instance.enumerate_physical_devices() }
        .ok()
        .and_then(|devices| {
            devices.into_iter().find_map(|device| {
                let properties = unsafe { instance.get_physical_device_properties(device) };
                (properties.api_version >= vk::API_VERSION_1_1
                    && properties.device_type != vk::PhysicalDeviceType::CPU)
                    .then(|| {
                        unsafe { std::ffi::CStr::from_ptr(properties.device_name.as_ptr()) }
                            .to_string_lossy()
                            .into_owned()
                    })
            })
        });
    unsafe { instance.destroy_instance(None) };
    found
}

pub fn install_runtime(package: &Package, log: &Log) -> Result<(), String> {
    let mut system = vec![0u16; 260];
    let count = unsafe { GetSystemDirectoryW(system.as_mut_ptr(), system.len() as u32) } as usize;
    if count == 0 || count >= system.len() {
        return Err("Não foi possível localizar os componentes do Windows.".to_owned());
    }
    let powershell = PathBuf::from(String::from_utf16_lossy(&system[..count]))
        .join(r"WindowsPowerShell\v1.0\powershell.exe");
    let local: &OsStr = if package.installer_is_local() {
        package.installer.as_os_str()
    } else {
        OsStr::new("")
    };
    log.write(if local.is_empty() {
        "Instalação autorizada. Fonte oficial da Microsoft."
    } else {
        "Instalação autorizada. Instalador offline incluído."
    });
    let output = Command::new(powershell)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            include_str!("install_runtime.ps1"),
        ])
        .env("AINDA_VIVO_REDIST_PATH", local)
        .creation_flags(NO_CONSOLE)
        .output()
        .map_err(|error| format!("Não foi possível abrir o instalador: {error}"))?;
    log.write(&format!(
        "Instalador: {}. {}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ));
    if output.status.success() {
        Ok(())
    } else if output.status.code() == Some(30) {
        Err(
            "O componente foi instalado. Reinicie o Windows e abra o inicializador novamente."
                .to_owned(),
        )
    } else {
        Err("A instalação não foi concluída. Consulte o diagnóstico e tente novamente.".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_missing_msvc_dlls_offer_the_microsoft_installer() {
        let imports = vec!["kernel32.dll".to_owned(), "vcruntime140.dll".to_owned()];
        let missing = missing_dlls(&imports, |name| name != "vcruntime140.dll");
        assert_eq!(readiness(true, &missing), Readiness::NeedsRuntime);
        assert_eq!(readiness(true, &[]), Readiness::Ready);
        assert_eq!(readiness(false, &missing), Readiness::Blocked);
        assert_eq!(
            readiness(true, &["api-ms-win-crt-runtime-l1-1-0.dll"]),
            Readiness::Blocked
        );
        assert_eq!(
            readiness(true, &["unknown.dll", "vcruntime140.dll"]),
            Readiness::Blocked
        );
    }
}
