use raw_window_handle::HasWindowHandle;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const WINDOW_STATE_VERSION: u32 = 1;
const DEFAULT_MAIN_WINDOW_WIDTH: i32 = 1380;
const DEFAULT_MAIN_WINDOW_HEIGHT: i32 = 860;
const MIN_RESTORED_WINDOW_WIDTH: i32 = 1000;
const MIN_RESTORED_WINDOW_HEIGHT: i32 = 650;

#[derive(Clone, Copy, Debug)]
pub struct MainWindowHandle {
    #[cfg(target_os = "windows")]
    hwnd: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MainWindowState {
    version: u32,
    monitor: String,
    offset_x: i32,
    offset_y: i32,
    width: i32,
    height: i32,
    maximized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkArea {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WindowRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

pub fn capture_main_window(source: &impl HasWindowHandle) -> Result<MainWindowHandle, String> {
    platform::capture_main_window(source)
}

/// Configures per-monitor DPI awareness before eframe creates the native window.
pub fn configure_process_dpi_awareness() -> Result<(), String> {
    platform::configure_process_dpi_awareness()
}

/// Restores the main window from the WSDD-owned state file, or maximizes it on first launch.
pub fn initialize_main_window(owner_window: MainWindowHandle) -> Result<(), String> {
    platform::initialize_main_window(owner_window, &main_window_state_path())
}

/// Saves monitor-relative restored bounds and maximized state for the main window.
pub fn save_main_window(owner_window: MainWindowHandle) -> Result<(), String> {
    platform::save_main_window(owner_window, &main_window_state_path())
}

fn main_window_state_path() -> PathBuf {
    crate::config::environment::path_config()
        .environment_root()
        .join("main-window.json")
}

fn clamped_window_rect(work: &WorkArea, state: &MainWindowState) -> WindowRect {
    let work_width = work.right - work.left;
    let work_height = work.bottom - work.top;
    let width = state
        .width
        .clamp(MIN_RESTORED_WINDOW_WIDTH.min(work_width), work_width);
    let height = state
        .height
        .clamp(MIN_RESTORED_WINDOW_HEIGHT.min(work_height), work_height);
    let x = (work.left + state.offset_x).clamp(work.left, work.right - width);
    let y = (work.top + state.offset_y).clamp(work.top, work.bottom - height);

    WindowRect {
        x,
        y,
        width,
        height,
    }
}

fn preserve_stable_restored_size(previous: &MainWindowState, current: &mut MainWindowState) {
    if previous.version == current.version
        && previous.monitor == current.monitor
        && !previous.maximized
        && !current.maximized
        && (previous.width - current.width).abs() <= 4
        && (previous.height - current.height).abs() <= 4
    {
        current.width = previous.width;
        current.height = previous.height;
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use raw_window_handle::RawWindowHandle;
    use std::{fs, mem::size_of, path::Path};
    use windows::Win32::{
        Foundation::{BOOL, HWND, LPARAM, RECT},
        Graphics::Gdi::{
            EnumDisplayMonitors, GetMonitorInfoW, MonitorFromWindow, HDC, HMONITOR, MONITORINFOEXW,
            MONITOR_DEFAULTTOPRIMARY,
        },
        UI::{
            HiDpi::{
                GetDpiForMonitor, GetDpiForWindow, SetProcessDpiAwarenessContext,
                SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT,
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, MDT_EFFECTIVE_DPI,
            },
            WindowsAndMessaging::{
                GetWindowPlacement, IsWindow, SetWindowPos, ShowWindow, HWND_TOP, SW_MAXIMIZE,
                SW_SHOWNORMAL, WINDOWPLACEMENT, WINDOWPLACEMENT_FLAGS,
            },
        },
    };

    #[derive(Clone)]
    struct MonitorDetails {
        device: String,
        work: WorkArea,
    }

    pub fn capture_main_window(source: &impl HasWindowHandle) -> Result<MainWindowHandle, String> {
        let raw_window = source.window_handle().map_err(|error| {
            format!("No se pudo obtener el handle nativo de la ventana de WSDD: {error}")
        })?;

        match raw_window.as_raw() {
            RawWindowHandle::Win32(handle) => Ok(MainWindowHandle {
                hwnd: handle.hwnd.get() as usize,
            }),
            _ => Err("WSDD requiere una ventana Win32 válida para restaurar tamaño.".to_owned()),
        }
    }

    pub fn configure_process_dpi_awareness() -> Result<(), String> {
        let _ =
            unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        Ok(())
    }

    pub fn initialize_main_window(
        owner_window: MainWindowHandle,
        state_path: &Path,
    ) -> Result<(), String> {
        let hwnd = checked_hwnd(owner_window)?;
        let restored = fs::read_to_string(state_path)
            .ok()
            .and_then(|json| serde_json::from_str::<MainWindowState>(&json).ok())
            .filter(|state| state.version == WINDOW_STATE_VERSION)
            .and_then(|state| restore_window_state(hwnd, &state).ok())
            .is_some();

        if !restored {
            let primary =
                monitor_details(unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) })?;
            let work_width = primary.work.right - primary.work.left;
            let work_height = primary.work.bottom - primary.work.top;
            let width = DEFAULT_MAIN_WINDOW_WIDTH.min(work_width);
            let height = DEFAULT_MAIN_WINDOW_HEIGHT.min(work_height);
            let x = primary.work.left + ((work_width - width) / 2);
            let y = primary.work.top + ((work_height - height) / 2);
            unsafe {
                SetWindowPos(hwnd, HWND_TOP, x, y, width, height, Default::default()).map_err(
                    |error| format!("No se pudo posicionar la ventana principal: {error}"),
                )?;
                let _ = ShowWindow(hwnd, SW_MAXIMIZE);
            }
        }

        Ok(())
    }

    pub fn save_main_window(
        owner_window: MainWindowHandle,
        state_path: &Path,
    ) -> Result<(), String> {
        let _dpi_guard = DpiAwarenessGuard::per_monitor_v2();
        let hwnd = checked_hwnd(owner_window)?;
        let mut placement = WINDOWPLACEMENT {
            length: size_of::<WINDOWPLACEMENT>() as u32,
            flags: WINDOWPLACEMENT_FLAGS(0),
            ..Default::default()
        };
        unsafe { GetWindowPlacement(hwnd, &mut placement) }
            .map_err(|error| format!("No se pudo leer la posicion de la ventana: {error}"))?;

        let monitor =
            monitor_details(unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) })?;
        let rect = placement.rcNormalPosition;
        let raw_state = MainWindowState {
            version: WINDOW_STATE_VERSION,
            monitor: monitor.device,
            offset_x: rect.left - monitor.work.left,
            offset_y: rect.top - monitor.work.top,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
            maximized: placement.showCmd == SW_MAXIMIZE.0 as u32,
        };
        let clamped = clamped_window_rect(&monitor.work, &raw_state);
        let mut state = MainWindowState {
            offset_x: clamped.x - monitor.work.left,
            offset_y: clamped.y - monitor.work.top,
            width: clamped.width,
            height: clamped.height,
            ..raw_state
        };

        if let Some(previous) = fs::read_to_string(state_path)
            .ok()
            .and_then(|json| serde_json::from_str::<MainWindowState>(&json).ok())
        {
            preserve_stable_restored_size(&previous, &mut state);
        }

        let json = serde_json::to_string_pretty(&state)
            .map_err(|error| format!("No se pudo serializar el estado de ventana: {error}"))?;
        if fs::read_to_string(state_path)
            .map(|existing_json| existing_json == json)
            .unwrap_or(false)
        {
            return Ok(());
        }
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!("No se pudo crear el directorio de estado de ventana: {error}")
            })?;
        }
        fs::write(state_path, json)
            .map_err(|error| format!("No se pudo guardar el estado de ventana: {error}"))
    }

    fn checked_hwnd(owner_window: MainWindowHandle) -> Result<HWND, String> {
        let hwnd = HWND(owner_window.hwnd as *mut _);
        if unsafe { IsWindow(hwnd) }.as_bool() {
            Ok(hwnd)
        } else {
            Err("La ventana principal de WSDD ya no es valida.".to_owned())
        }
    }

    fn restore_window_state(hwnd: HWND, state: &MainWindowState) -> Result<(), String> {
        let _dpi_guard = DpiAwarenessGuard::per_monitor_v2();
        let monitor = match find_monitor(&state.monitor) {
            Some(monitor) => monitor,
            None if state.maximized => {
                monitor_details(unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) })?
            }
            None => return Err("El monitor guardado ya no esta disponible.".to_owned()),
        };
        let rect = clamped_window_rect(&monitor.work, state);
        let (width, height) =
            compensate_restore_size_for_current_dpi(hwnd, &monitor, rect.width, rect.height);

        unsafe {
            SetWindowPos(
                hwnd,
                HWND_TOP,
                rect.x,
                rect.y,
                width,
                height,
                Default::default(),
            )
            .map_err(|error| format!("No se pudo restaurar la ventana principal: {error}"))?;
            let _ = ShowWindow(
                hwnd,
                if state.maximized {
                    SW_MAXIMIZE
                } else {
                    SW_SHOWNORMAL
                },
            );
        }
        Ok(())
    }

    struct DpiAwarenessGuard {
        previous: DPI_AWARENESS_CONTEXT,
    }

    impl DpiAwarenessGuard {
        fn per_monitor_v2() -> Self {
            let previous =
                unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
            Self { previous }
        }
    }

    impl Drop for DpiAwarenessGuard {
        fn drop(&mut self) {
            let _ = unsafe { SetThreadDpiAwarenessContext(self.previous) };
        }
    }

    fn compensate_restore_size_for_current_dpi(
        hwnd: HWND,
        monitor: &MonitorDetails,
        width: i32,
        height: i32,
    ) -> (i32, i32) {
        let current_dpi = unsafe { GetDpiForWindow(hwnd) };
        let target_dpi = monitor_dpi(monitor).unwrap_or(current_dpi);
        if current_dpi == 0 || target_dpi == 0 || current_dpi == target_dpi {
            return (width, height);
        }

        (
            scale_i32(width, current_dpi, target_dpi),
            scale_i32(height, current_dpi, target_dpi),
        )
    }

    fn monitor_dpi(monitor: &MonitorDetails) -> Option<u32> {
        let handle = find_monitor_handle(&monitor.device)?;
        let mut dpi_x = 0;
        let mut dpi_y = 0;
        unsafe { GetDpiForMonitor(handle, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) }.ok()?;
        Some(dpi_x)
    }

    fn scale_i32(value: i32, numerator: u32, denominator: u32) -> i32 {
        ((i64::from(value) * i64::from(numerator) + i64::from(denominator / 2))
            / i64::from(denominator)) as i32
    }

    fn find_monitor(device: &str) -> Option<MonitorDetails> {
        unsafe extern "system" fn callback(
            monitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut RECT,
            data: LPARAM,
        ) -> BOOL {
            let context = unsafe { &mut *(data.0 as *mut (&str, Option<MonitorDetails>)) };
            if let Ok(details) = monitor_details(monitor) {
                if details.device == context.0 {
                    context.1 = Some(details);
                    return false.into();
                }
            }
            true.into()
        }

        let mut context = (device, None);
        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(callback),
                LPARAM((&mut context as *mut (&str, Option<MonitorDetails>)) as isize),
            );
        }
        context.1
    }

    fn find_monitor_handle(device: &str) -> Option<HMONITOR> {
        unsafe extern "system" fn callback(
            monitor: HMONITOR,
            _hdc: HDC,
            _rect: *mut RECT,
            data: LPARAM,
        ) -> BOOL {
            let context = unsafe { &mut *(data.0 as *mut (&str, Option<HMONITOR>)) };
            if let Ok(details) = monitor_details(monitor) {
                if details.device == context.0 {
                    context.1 = Some(monitor);
                    return false.into();
                }
            }
            true.into()
        }

        let mut context = (device, None);
        unsafe {
            let _ = EnumDisplayMonitors(
                None,
                None,
                Some(callback),
                LPARAM(&mut context as *mut _ as isize),
            );
        }
        context.1
    }

    fn monitor_details(handle: HMONITOR) -> Result<MonitorDetails, String> {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        if !unsafe { GetMonitorInfoW(handle, &mut info.monitorInfo as *mut _) }.as_bool() {
            return Err("No se pudo consultar el monitor de la ventana.".to_owned());
        }
        let device_len = info
            .szDevice
            .iter()
            .position(|value| *value == 0)
            .unwrap_or(info.szDevice.len());
        Ok(MonitorDetails {
            device: String::from_utf16_lossy(&info.szDevice[..device_len]),
            work: WorkArea::from(info.monitorInfo.rcWork),
        })
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;
    use std::path::Path;

    pub fn capture_main_window(_source: &impl HasWindowHandle) -> Result<MainWindowHandle, String> {
        Ok(MainWindowHandle {})
    }

    pub fn configure_process_dpi_awareness() -> Result<(), String> {
        Ok(())
    }

    pub fn initialize_main_window(
        _owner_window: MainWindowHandle,
        _state_path: &Path,
    ) -> Result<(), String> {
        Ok(())
    }

    pub fn save_main_window(
        _owner_window: MainWindowHandle,
        _state_path: &Path,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(target_os = "windows")]
impl From<windows::Win32::Foundation::RECT> for WorkArea {
    fn from(rect: windows::Win32::Foundation::RECT) -> Self {
        Self {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restored_bounds_stay_inside_negative_origin_monitor() {
        let work = WorkArea {
            left: -1920,
            top: 0,
            right: 0,
            bottom: 1032,
        };
        let state = MainWindowState {
            version: WINDOW_STATE_VERSION,
            monitor: r"\\.\DISPLAY5".to_owned(),
            offset_x: 220,
            offset_y: 100,
            width: 1100,
            height: 760,
            maximized: false,
        };

        assert_eq!(
            clamped_window_rect(&work, &state),
            WindowRect {
                x: -1700,
                y: 100,
                width: 1100,
                height: 760,
            }
        );
    }

    #[test]
    fn oversized_restored_bounds_are_clamped_to_work_area() {
        let work = WorkArea {
            left: 0,
            top: 0,
            right: 1536,
            bottom: 816,
        };
        let state = MainWindowState {
            version: WINDOW_STATE_VERSION,
            monitor: r"\\.\DISPLAY1".to_owned(),
            offset_x: 500,
            offset_y: 300,
            width: 3000,
            height: 2000,
            maximized: true,
        };

        assert_eq!(
            clamped_window_rect(&work, &state),
            WindowRect {
                x: 0,
                y: 0,
                width: 1536,
                height: 816,
            }
        );
    }

    #[test]
    fn preserves_previous_restored_size_when_maximized_state_is_saved() {
        let previous = MainWindowState {
            version: WINDOW_STATE_VERSION,
            monitor: r"\\.\DISPLAY1".to_owned(),
            offset_x: 100,
            offset_y: 100,
            width: 1200,
            height: 780,
            maximized: false,
        };
        let mut current = MainWindowState {
            version: WINDOW_STATE_VERSION,
            monitor: r"\\.\DISPLAY1".to_owned(),
            offset_x: 102,
            offset_y: 101,
            width: 1203,
            height: 778,
            maximized: false,
        };

        preserve_stable_restored_size(&previous, &mut current);

        assert_eq!(current.width, 1200);
        assert_eq!(current.height, 780);
    }

    #[test]
    fn maximized_state_does_not_overwrite_restored_size() {
        let previous = MainWindowState {
            version: WINDOW_STATE_VERSION,
            monitor: r"\\.\DISPLAY1".to_owned(),
            offset_x: 100,
            offset_y: 100,
            width: 1200,
            height: 780,
            maximized: false,
        };
        let mut current = MainWindowState {
            version: WINDOW_STATE_VERSION,
            monitor: r"\\.\DISPLAY1".to_owned(),
            offset_x: 0,
            offset_y: 0,
            width: 1536,
            height: 816,
            maximized: true,
        };

        preserve_stable_restored_size(&previous, &mut current);

        assert_eq!(current.width, 1536);
        assert_eq!(current.height, 816);
    }
}
