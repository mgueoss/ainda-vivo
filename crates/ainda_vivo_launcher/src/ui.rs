use crate::{
    check::{Assessment, Log, Package, Readiness, install_runtime},
    wide,
};
use std::{
    io,
    process::Child,
    ptr::{null, null_mut},
    sync::mpsc::{self, Receiver},
};
use windows_sys::Win32::{
    Foundation::{COLORREF, HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Controls::{
            DRAWITEMSTRUCT, ICC_PROGRESS_CLASS, INITCOMMONCONTROLSEX, InitCommonControlsEx,
            ODS_DISABLED, ODS_FOCUS, ODS_SELECTED, PBM_SETBARCOLOR, PBM_SETBKCOLOR, PBM_SETPOS,
            PROGRESS_CLASSW,
        },
        Input::KeyboardAndMouse::{EnableWindow, SetFocus},
        Shell::ShellExecuteW,
        WindowsAndMessaging::*,
    },
};
use windows_sys::core::w;

const BG: COLORREF = 0x171b1b;
const TEXT: COLORREF = 0xdbe5dd;
const MUTED: COLORREF = 0xadbdb4;
const GREEN: COLORREF = 0x9ad2ae;
const ID_PLAY: u16 = 101;
const ID_LOG: u16 = 102;
const ID_STATUS: u16 = 103;
const ID_GPU: u16 = 104;
const ID_DEPS: u16 = 105;
const ID_TITLE: u16 = 106;
const WIDTH: i32 = 576;
const HEIGHT: i32 = 360;
const TIMER: usize = 1;

type Worker = Receiver<Result<Assessment, String>>;

struct Controls {
    play: HWND,
    status: HWND,
    gpu: HWND,
    dependencies: HWND,
    progress: HWND,
}

struct Ui {
    pkg: Package,
    log: Log,
    controls: Option<Controls>,
    state: Readiness,
    worker: Option<Worker>,
    auto_launch: bool,
    game: Option<Child>,
}

impl Ui {
    fn scan(&mut self, repair: bool) {
        let (send, receive) = mpsc::channel();
        let pkg = self.pkg.clone();
        let log = self.log.clone();
        self.worker = Some(receive);
        self.auto_launch = repair;
        std::thread::spawn(move || {
            let result = if repair {
                install_runtime(&pkg, &log)
            } else {
                Ok(())
            };
            let result = result.map(|()| Assessment::inspect(&pkg));
            if let Ok(assessment) = &result {
                log.write(&format!(
                    "Verificação v{}: {:?}\n{}",
                    pkg.version, assessment.state, assessment.details
                ));
            }
            let _ = send.send(result);
        });
    }

    fn tick(&mut self) {
        if let Some(worker) = &self.worker {
            let value = match worker.try_recv() {
                Ok(value) => value,
                Err(mpsc::TryRecvError::Empty) => return,
                Err(mpsc::TryRecvError::Disconnected) => Err(
                    "A verificação foi interrompida. Abra o inicializador novamente.".to_owned(),
                ),
            };
            self.worker = None;
            let ctl = self.controls.as_ref().unwrap();
            match value {
                Ok(value) => {
                    self.state = value.state;
                    label(ctl.status, &value.status);
                    label(ctl.gpu, &value.compatibility);
                    label(ctl.dependencies, &value.dependencies);
                    progress(
                        ctl.progress,
                        if self.state == Readiness::Ready {
                            100
                        } else {
                            75
                        },
                    );
                    unsafe {
                        EnableWindow(ctl.play, (self.state != Readiness::Blocked).into());
                    }
                    if self.auto_launch && self.state == Readiness::Ready {
                        self.play();
                    }
                }
                Err(error) => {
                    self.log.write(&error);
                    label(ctl.status, &error);
                    progress(ctl.progress, 75);
                    unsafe {
                        EnableWindow(ctl.play, 1);
                    }
                }
            }
            return;
        }
        if let Some(child) = self.game.as_mut()
            && let Ok(Some(status)) = child.try_wait()
        {
            self.game = None;
            let ctl = self.controls.as_ref().unwrap();
            self.log.write(&format!("O jogo encerrou com {status}."));
            label(
                ctl.status,
                if status.success() {
                    "Sessão encerrada. Clique em JOGAR para abrir novamente."
                } else {
                    "O jogo encerrou com erro. Abra DIAGNÓSTICO para consultar os registros."
                },
            );
            unsafe {
                EnableWindow(ctl.play, 1);
            }
        }
    }

    fn play(&mut self) {
        let ctl = self.controls.as_ref().unwrap();
        match self.log.launch(&self.pkg) {
            Ok(child) => {
                self.game = Some(child);
                label(ctl.status, "Jogo em execução.");
                unsafe {
                    EnableWindow(ctl.play, 0);
                }
            }
            Err(error) => {
                self.log.write(&format!("Erro ao abrir o jogo: {error}"));
                label(
                    ctl.status,
                    "Não foi possível abrir o jogo. Consulte o diagnóstico.",
                );
            }
        }
    }

    fn begin_repair(&mut self) {
        let ctl = self.controls.as_ref().unwrap();
        unsafe {
            EnableWindow(ctl.play, 0);
        }
        label(
            ctl.status,
            "Preparando o runtime Microsoft Visual C++ x64...",
        );
        progress(ctl.progress, 45);
        self.scan(true);
    }
}

fn label(hwnd: HWND, text: &str) {
    unsafe {
        SetWindowTextW(hwnd, wide(text).as_ptr());
    }
}

fn progress(hwnd: HWND, value: usize) {
    unsafe {
        SendMessageW(hwnd, PBM_SETPOS, value, 0);
    }
}

unsafe fn control(
    hwnd: HWND,
    class: *const u16,
    text: &str,
    style: u32,
    id: u16,
    rect: [i32; 4],
    font: HFONT,
) -> HWND {
    let [x, y, width, height] = rect;
    let handle = unsafe {
        CreateWindowExW(
            0,
            class,
            wide(text).as_ptr(),
            WS_CHILD | WS_VISIBLE | style,
            x,
            y,
            width,
            height,
            hwnd,
            id as usize as HMENU,
            GetModuleHandleW(null()),
            null(),
        )
    };
    if !font.is_null() {
        unsafe {
            SendMessageW(handle, WM_SETFONT, font as WPARAM, 1);
        }
    }
    handle
}

fn font(size: i32, weight: i32, name: *const u16) -> HFONT {
    unsafe { CreateFontW(-size, 0, 0, 0, weight, 0, 0, 0, 1, 0, 0, 5, 0, name) }
}

pub fn run(pkg: Package, log: Log) -> io::Result<()> {
    let title = format!("AINDA VIVO | Preparação v{}", pkg.version);
    let mut ui = Ui {
        pkg,
        log,
        controls: None,
        state: Readiness::Blocked,
        worker: None,
        auto_launch: false,
        game: None,
    };
    let mut bounds = RECT {
        left: 0,
        top: 0,
        right: WIDTH,
        bottom: HEIGHT,
    };
    let style = WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX;
    let ex_style = WS_EX_CONTROLPARENT;
    unsafe {
        let instance = GetModuleHandleW(null());
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: w!("AindaVivoLauncher"),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err(io::Error::last_os_error());
        }
        InitCommonControlsEx(&INITCOMMONCONTROLSEX {
            dwSize: size_of::<INITCOMMONCONTROLSEX>() as u32,
            dwICC: ICC_PROGRESS_CLASS,
        });
        AdjustWindowRectEx(&mut bounds, style, 0, ex_style);
        let hwnd = CreateWindowExW(
            ex_style,
            class.lpszClassName,
            wide(title).as_ptr(),
            style,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            bounds.right - bounds.left,
            bounds.bottom - bounds.top,
            null_mut(),
            null_mut(),
            instance,
            (&mut ui as *mut Ui).cast(),
        );
        if hwnd.is_null() {
            return Err(io::Error::last_os_error());
        }
        let body = font(16, 400, w!("Segoe UI"));
        let heading = font(32, 650, w!("Segoe UI"));
        let mono = font(15, 400, w!("Consolas"));
        control(
            hwnd,
            w!("STATIC"),
            "AINDA VIVO",
            0,
            ID_TITLE,
            [32, 25, 512, 43],
            heading,
        );
        control(
            hwnd,
            w!("STATIC"),
            &format!("VERSÃO INSTALADA  v{}", ui.pkg.version),
            0,
            0,
            [34, 74, 506, 21],
            mono,
        );
        let dependencies = control(
            hwnd,
            w!("STATIC"),
            "Verificando dependências...",
            0,
            ID_DEPS,
            [34, 125, 508, 43],
            body,
        );
        let gpu = control(
            hwnd,
            w!("STATIC"),
            "Verificando Windows e GPU...",
            0,
            ID_GPU,
            [34, 171, 508, 48],
            body,
        );
        let bar = control(
            hwnd,
            PROGRESS_CLASSW,
            "",
            0,
            0,
            [34, 225, 508, 9],
            null_mut(),
        );
        SendMessageW(bar, PBM_SETBARCOLOR, 0, GREEN as LPARAM);
        SendMessageW(bar, PBM_SETBKCOLOR, 0, 0x333a36);
        progress(bar, 20);
        let status = control(
            hwnd,
            w!("STATIC"),
            "Preparando a sessão.",
            0,
            ID_STATUS,
            [34, 247, 508, 44],
            body,
        );
        let play = control(
            hwnd,
            w!("BUTTON"),
            "JOGAR",
            WS_TABSTOP | BS_OWNERDRAW as u32,
            ID_PLAY,
            [34, 302, 172, 36],
            body,
        );
        let log_button = control(
            hwnd,
            w!("BUTTON"),
            "DIAGNÓSTICO",
            WS_TABSTOP | BS_OWNERDRAW as u32,
            ID_LOG,
            [380, 302, 162, 36],
            body,
        );
        EnableWindow(play, 0);
        ui.controls = Some(Controls {
            play,
            status,
            gpu,
            dependencies,
            progress: bar,
        });
        ui.log.write("Inicializador aberto.");
        ui.scan(false);
        SetTimer(hwnd, TIMER, 120, None);
        ShowWindow(hwnd, SW_SHOW);
        SetFocus(log_button);
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if IsDialogMessageW(hwnd, &msg) == 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        DeleteObject(body);
        DeleteObject(heading);
        DeleteObject(mono);
    }
    Ok(())
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        let state = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Ui;
        match message {
            WM_NCCREATE => {
                let create = &*(lparam as *const CREATESTRUCTW);
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
            }
            WM_ERASEBKGND => {
                let hdc = wparam as HDC;
                let mut bounds = RECT::default();
                GetClientRect(hwnd, &mut bounds);
                fill(hdc, &bounds, BG);
                return 1;
            }
            WM_CTLCOLORSTATIC => {
                let hdc = wparam as HDC;
                let id = GetDlgCtrlID(lparam as HWND) as u16;
                SetTextColor(hdc, if id == ID_TITLE { TEXT } else { MUTED });
                SetBkColor(hdc, BG);
                let brush = GetStockObject(DC_BRUSH);
                SetDCBrushColor(hdc, BG);
                return brush as LRESULT;
            }
            WM_DRAWITEM => {
                let draw = &*(lparam as *const DRAWITEMSTRUCT);
                if matches!(draw.CtlID as u16, ID_PLAY | ID_LOG) {
                    button(draw);
                    return 1;
                }
            }
            WM_TIMER if wparam == TIMER && !state.is_null() => {
                (*state).tick();
                return 0;
            }
            WM_COMMAND if !state.is_null() && (wparam >> 16) & 0xffff == 0 => {
                match (wparam & 0xffff) as u16 {
                    ID_PLAY if (*state).worker.is_none() && (*state).game.is_none() => {
                        match (*state).state {
                            Readiness::Ready => (*state).play(),
                            Readiness::NeedsRuntime => {
                                let message = if (*state).pkg.installer_is_local() {
                                    "Autoriza instalar o runtime Visual C++ x64 incluído neste pacote?"
                                } else {
                                    "Autoriza baixar da Microsoft e instalar o runtime Visual C++ x64? A instalação pode solicitar permissão de administrador."
                                };
                                if MessageBoxW(
                                    hwnd,
                                    wide(message).as_ptr(),
                                    w!("Microsoft Visual C++ x64"),
                                    MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
                                ) == IDYES
                                {
                                    (*state).begin_repair();
                                }
                            }
                            Readiness::Blocked => (),
                        }
                    }
                    ID_LOG => {
                        let path = wide(&(*state).log.path);
                        let result = ShellExecuteW(
                            hwnd,
                            w!("open"),
                            path.as_ptr(),
                            null(),
                            null(),
                            SW_SHOWNORMAL,
                        );
                        if result as isize <= 32 {
                            MessageBoxW(
                                hwnd,
                                path.as_ptr(),
                                w!("Caminho do diagnóstico"),
                                MB_ICONINFORMATION,
                            );
                        }
                    }
                    _ => (),
                }
                return 0;
            }
            WM_DESTROY => {
                KillTimer(hwnd, TIMER);
                PostQuitMessage(0);
                return 0;
            }
            _ => (),
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }
}

unsafe fn fill(hdc: HDC, bounds: &RECT, color: COLORREF) {
    unsafe {
        SetDCBrushColor(hdc, color);
        FillRect(hdc, bounds, GetStockObject(DC_BRUSH));
    }
}

unsafe fn button(draw: &DRAWITEMSTRUCT) {
    let primary = draw.CtlID as u16 == ID_PLAY;
    let disabled = draw.itemState & ODS_DISABLED != 0;
    let pressed = draw.itemState & ODS_SELECTED != 0;
    let theme = if disabled {
        0x292f2c
    } else if pressed {
        0x729a80
    } else if primary {
        GREEN
    } else {
        0x323c36
    };
    unsafe {
        fill(draw.hDC, &draw.rcItem, theme);
        let mut text = [0u16; 48];
        let count = GetWindowTextW(draw.hwndItem, text.as_mut_ptr(), text.len() as i32);
        let mut rect = draw.rcItem;
        SetBkMode(draw.hDC, TRANSPARENT as i32);
        SetTextColor(draw.hDC, if primary && !disabled { 0x17211b } else { TEXT });
        let font = SendMessageW(draw.hwndItem, WM_GETFONT, 0, 0) as HFONT;
        let previous = SelectObject(draw.hDC, font);
        DrawTextW(
            draw.hDC,
            text.as_ptr(),
            count,
            &mut rect,
            DT_SINGLELINE | DT_CENTER | DT_VCENTER,
        );
        SelectObject(draw.hDC, previous);
        if draw.itemState & ODS_FOCUS != 0 {
            InflateRect(&mut rect, -4, -4);
            DrawFocusRect(draw.hDC, &rect);
        }
    }
}
