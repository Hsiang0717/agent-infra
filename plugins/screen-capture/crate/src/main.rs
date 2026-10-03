use std::env;
use std::ffi::c_void;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::channel;
use std::time::Duration;

use windows::core::{factory, s, BOOL, Interface, Result, PWSTR};
use windows::Foundation::TypedEventHandler;
use windows::Graphics::Capture::{Direct3D11CaptureFramePool, GraphicsCaptureItem};
use windows::Graphics::DirectX::Direct3D11::IDirect3DDevice;
use windows::Graphics::DirectX::DirectXPixelFormat;
use windows::Win32::Foundation::{CloseHandle, HMODULE, HWND, LPARAM, POINT, RECT, TRUE};
use windows::Win32::Graphics::Direct3D::{
    D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1,
    D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11CreateDevice, ID3D11Device, ID3D11DeviceContext, ID3D11Texture2D,
    D3D11_CPU_ACCESS_READ, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE,
    D3D11_MAP_READ, D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
use windows::Win32::Graphics::Dxgi::IDXGIDevice;
use windows::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    HDC, MonitorFromPoint, ReleaseDC, SelectObject, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, MONITOR_DEFAULTTOPRIMARY, SRCCOPY,
};
use windows::Win32::System::Console::GetConsoleWindow;
use windows::Win32::System::StationsAndDesktops::{
    CloseDesktop, EnumDesktopWindows, OpenDesktopA, SetThreadDesktop, DESKTOP_CONTROL_FLAGS,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::System::WinRT::Direct3D11::{
    CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess,
};
use windows::Win32::System::WinRT::Graphics::Capture::IGraphicsCaptureItemInterop;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetSystemMetrics, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    ShowWindow, ShowWindowAsync, SM_CXSCREEN, SM_CYSCREEN, SW_HIDE, SW_RESTORE, SW_SHOW,
};

#[link(name = "user32")]
unsafe extern "system" {
    fn PrintWindow(hwnd: HWND, hdc: HDC, nflags: u32) -> BOOL;
}

#[derive(Debug, Clone)]
struct WindowInfo {
    hwnd: HWND,
    pid: u32,
    process_name: String,
    title: String,
    rect: RECT,
}

#[derive(Debug, Clone)]
enum TargetMode {
    ByProcessName(String),
    ByTitle(String),
    ByPid(u32),
    FullScreen,
}

struct CliOptions {
    target: TargetMode,
    output_path: PathBuf,
    no_border: bool,
}

struct ConsoleGuard(HWND);

impl Drop for ConsoleGuard {
    fn drop(&mut self) {
        if self.0 != HWND::default() {
            unsafe {
                let _ = ShowWindow(self.0, SW_SHOW);
            }
        }
    }
}

fn print_agent_stdout(msg: &str) {
    println!("{}", msg);
}

fn print_agent_stderr(msg: &str) {
    eprintln!("{}", msg);
}

fn get_process_name_by_pid(pid: u32) -> String {
    unsafe {
        let process_handle = OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        );
        if let Ok(handle) = process_handle {
            let mut name_buf = [0u16; 1024];
            let mut len = name_buf.len() as u32;
            let mut result = String::from("unknown");
            if QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), PWSTR(name_buf.as_mut_ptr()), &mut len).is_ok() && len > 0 {
                let full = String::from_utf16_lossy(&name_buf[..len as usize]);
                if let Some(fname) = Path::new(&full).file_name() {
                    result = fname.to_string_lossy().to_string();
                }
            }
            let _ = CloseHandle(handle);
            return result;
        }
    }
    String::from("unknown")
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    unsafe {
        let list = &mut *(lparam.0 as *mut Vec<WindowInfo>);

        if !IsWindowVisible(hwnd).as_bool() {
            return TRUE;
        }

        let mut rect = RECT::default();
        if GetWindowRect(hwnd, &mut rect).is_err() {
            return TRUE;
        }
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 30 || height <= 30 {
            return TRUE;
        }

        let mut cloaked: u32 = 0;
        let hr = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &mut cloaked as *mut u32 as *mut c_void,
            std::mem::size_of::<u32>() as u32,
        );
        if hr.is_ok() && cloaked != 0 {
            return TRUE;
        }

        let mut buf = [0u16; 512];
        let copied = GetWindowTextW(hwnd, &mut buf);
        let title = String::from_utf16_lossy(&buf[..copied as usize]).trim().to_string();
        if title.is_empty() || title == "Default IME" || title == "MSCTFIME UI" || title == "Program Manager" {
            return TRUE;
        }

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let proc_name = get_process_name_by_pid(pid);

        list.push(WindowInfo {
            hwnd,
            pid,
            process_name: proc_name,
            title,
            rect,
        });

        TRUE
    }
}

fn collect_all_windows() -> Vec<WindowInfo> {
    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        let desk = OpenDesktopA(s!("Default"), DESKTOP_CONTROL_FLAGS(0), false, 0x01FF);
        if let Ok(hdesk) = desk {
            let _ = SetThreadDesktop(hdesk);
            let lparam = LPARAM(&mut windows as *mut Vec<WindowInfo> as isize);
            let _ = EnumDesktopWindows(Some(hdesk), Some(enum_windows_proc), lparam);
            let _ = CloseDesktop(hdesk);
        } else {
            let lparam = LPARAM(&mut windows as *mut Vec<WindowInfo> as isize);
            let _ = EnumWindows(Some(enum_windows_proc), lparam);
        }
    }
    windows
}

fn parse_cli_args() -> std::result::Result<CliOptions, String> {
    let args: Vec<String> = env::args().collect();
    let mut target = None;
    let mut output_path = None;
    let mut no_border = true;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--name" | "-ProcessName" => {
                i += 1;
                if i >= args.len() {
                    return Err(String::from("Missing value for --name"));
                }
                target = Some(TargetMode::ByProcessName(args[i].clone()));
            }
            "--title" | "-WindowTitle" => {
                i += 1;
                if i >= args.len() {
                    return Err(String::from("Missing value for --title"));
                }
                target = Some(TargetMode::ByTitle(args[i].clone()));
            }
            "--pid" | "-Id" | "-PID" => {
                i += 1;
                if i >= args.len() {
                    return Err(String::from("Missing value for --pid"));
                }
                let pid = args[i].parse::<u32>().map_err(|_| format!("Invalid PID: {}", args[i]))?;
                target = Some(TargetMode::ByPid(pid));
            }
            "--fullscreen" | "-FullScreen" => {
                target = Some(TargetMode::FullScreen);
            }
            "--out" | "-OutputPath" => {
                i += 1;
                if i >= args.len() {
                    return Err(String::from("Missing value for --out"));
                }
                output_path = Some(PathBuf::from(&args[i]));
            }
            "--no-border" => {
                no_border = true;
            }
            "--with-border" => {
                no_border = false;
            }
            "-h" | "--help" => {
                return Err(String::from(
                    "Usage: capture-screen.exe [--name <process>] [--title <title>] [--pid <id>] [--fullscreen] [--out <path>] [--no-border]"
                ));
            }
            other => {
                return Err(format!("Unrecognized option: {}", other));
            }
        }
        i += 1;
    }

    let target = target.unwrap_or(TargetMode::FullScreen);
    let out = output_path.unwrap_or_else(|| PathBuf::from("screenshot.png"));

    Ok(CliOptions {
        target,
        output_path: out,
        no_border,
    })
}

fn create_d3d_device() -> Result<(ID3D11Device, ID3D11DeviceContext, IDirect3DDevice)> {
    let mut device: Option<ID3D11Device> = None;
    let mut context: Option<ID3D11DeviceContext> = None;

    let feature_levels = [
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
    ];

    unsafe {
        let hr = D3D11CreateDevice(
            None,
            D3D_DRIVER_TYPE_HARDWARE,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            Some(&feature_levels),
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            Some(&mut context),
        );

        if hr.is_err() {
            D3D11CreateDevice(
                None,
                D3D_DRIVER_TYPE_WARP,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_BGRA_SUPPORT,
                Some(&feature_levels),
                D3D11_SDK_VERSION,
                Some(&mut device),
                None,
                Some(&mut context),
            )?;
        }

        let d3d_device = device.unwrap();
        let d3d_context = context.unwrap();

        let dxgi_device: IDXGIDevice = d3d_device.cast()?;
        let inspectable = CreateDirect3D11DeviceFromDXGIDevice(&dxgi_device)?;
        let winrt_device: IDirect3DDevice = inspectable.cast()?;

        Ok((d3d_device, d3d_context, winrt_device))
    }
}

fn capture_wgc(
    item: &GraphicsCaptureItem,
    out_path: &Path,
    no_border: bool,
) -> Result<(u32, u32)> {
    let (d3d_device, d3d_context, winrt_device) = create_d3d_device()?;
    let item_size = item.Size()?;
    let width = item_size.Width as u32;
    let height = item_size.Height as u32;

    if width == 0 || height == 0 {
        return Err(windows::core::Error::new(
            windows::core::HRESULT(-1),
            "Target capture item has 0 width or height",
        ));
    }

    let frame_pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &winrt_device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        item_size,
    )?;

    let session = frame_pool.CreateCaptureSession(item)?;
    if no_border {
        let _ = session.SetIsBorderRequired(false);
    }
    let _ = session.SetIsCursorCaptureEnabled(false);

    let (tx, rx) = channel::<()>();

    frame_pool.FrameArrived(&TypedEventHandler::new(
        move |_, _| {
            let _ = tx.send(());
            Ok(())
        },
    ))?;

    session.StartCapture()?;

    let _ = rx
        .recv_timeout(Duration::from_millis(2500))
        .map_err(|e| windows::core::Error::new(windows::core::HRESULT(-1), format!("Frame arrival timeout: {}", e)))?;

    let frame = frame_pool.TryGetNextFrame()?;
    let surface = frame.Surface()?;
    let access: IDirect3DDxgiInterfaceAccess = surface.cast()?;
    let d3d_texture: ID3D11Texture2D = unsafe { access.GetInterface()? };

    let mut tex_desc = D3D11_TEXTURE2D_DESC::default();
    unsafe {
        d3d_texture.GetDesc(&mut tex_desc);
    }

    let cap_width = tex_desc.Width;
    let cap_height = tex_desc.Height;

    let staging_desc = D3D11_TEXTURE2D_DESC {
        Width: cap_width,
        Height: cap_height,
        MipLevels: 1,
        ArraySize: 1,
        Format: tex_desc.Format,
        SampleDesc: DXGI_SAMPLE_DESC { Count: 1, Quality: 0 },
        Usage: D3D11_USAGE_STAGING,
        BindFlags: 0,
        CPUAccessFlags: D3D11_CPU_ACCESS_READ.0 as u32,
        MiscFlags: 0,
    };

    let mut staging_texture: Option<ID3D11Texture2D> = None;
    unsafe {
        d3d_device.CreateTexture2D(&staging_desc, None, Some(&mut staging_texture))?;
    }
    let staging_texture = staging_texture.unwrap();

    unsafe {
        d3d_context.CopyResource(&staging_texture, &d3d_texture);

        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        d3d_context.Map(&staging_texture, 0, D3D11_MAP_READ, 0, Some(&mut mapped))?;

        let row_pitch = mapped.RowPitch as usize;
        let mut rgba_buffer = vec![0u8; (cap_width * cap_height * 4) as usize];

        let src_ptr = mapped.pData as *const u8;
        for y in 0..cap_height as usize {
            let src_row = src_ptr.add(y * row_pitch);
            let dest_row_offset = y * (cap_width as usize) * 4;

            for x in 0..cap_width as usize {
                let px_src = src_row.add(x * 4);
                let px_dst = dest_row_offset + x * 4;

                let b = *px_src;
                let g = *px_src.add(1);
                let r = *px_src.add(2);
                let a = *px_src.add(3);

                rgba_buffer[px_dst] = r;
                rgba_buffer[px_dst + 1] = g;
                rgba_buffer[px_dst + 2] = b;
                rgba_buffer[px_dst + 3] = if a == 0 { 255 } else { a };
            }
        }

        d3d_context.Unmap(&staging_texture, 0);

        let _ = session.Close();
        let _ = frame_pool.Close();

        image::save_buffer(
            out_path,
            &rgba_buffer,
            cap_width,
            cap_height,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| windows::core::Error::new(windows::core::HRESULT(-1), format!("Image encode error: {}", e)))?;
    }

    Ok((cap_width, cap_height))
}

fn capture_gdi_window(hwnd: HWND, out_path: &Path) -> Result<(u32, u32)> {
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(hwnd, &mut rect)?;
        let width = (rect.right - rect.left).max(1) as i32;
        let height = (rect.bottom - rect.top).max(1) as i32;

        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        let old_bm = SelectObject(hdc_mem, hbm.into());

        // PW_RENDERFULLCONTENT = 2
        let printed = PrintWindow(hwnd, hdc_mem, 2);
        if !printed.as_bool() {
            // fallback BitBlt
            let hdc_wnd = GetDC(Some(hwnd));
            let _ = BitBlt(hdc_mem, 0, 0, width, height, Some(hdc_wnd), 0, 0, SRCCOPY);
            ReleaseDC(Some(hwnd), hdc_wnd);
        }

        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bgra_buf = vec![0u8; (width * height * 4) as usize];
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            height as u32,
            Some(bgra_buf.as_mut_ptr() as *mut c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old_bm);
        let _ = DeleteObject(hbm.into());
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        let mut rgba_buf = vec![0u8; bgra_buf.len()];
        for i in 0..(width * height) as usize {
            let b = bgra_buf[i * 4];
            let g = bgra_buf[i * 4 + 1];
            let r = bgra_buf[i * 4 + 2];
            let a = bgra_buf[i * 4 + 3];

            rgba_buf[i * 4] = r;
            rgba_buf[i * 4 + 1] = g;
            rgba_buf[i * 4 + 2] = b;
            rgba_buf[i * 4 + 3] = if a == 0 { 255 } else { a };
        }

        image::save_buffer(
            out_path,
            &rgba_buf,
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| windows::core::Error::new(windows::core::HRESULT(-1), format!("GDI Save error: {}", e)))?;

        Ok((width as u32, height as u32))
    }
}

fn capture_gdi_screen(out_path: &Path) -> Result<(u32, u32)> {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);

        let hdc_screen = GetDC(None);
        let hdc_mem = CreateCompatibleDC(Some(hdc_screen));
        let hbm = CreateCompatibleBitmap(hdc_screen, width, height);
        let old_bm = SelectObject(hdc_mem, hbm.into());

        BitBlt(hdc_mem, 0, 0, width, height, Some(hdc_screen), 0, 0, SRCCOPY)?;

        let mut bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width,
                biHeight: -height, // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bgra_buf = vec![0u8; (width * height * 4) as usize];
        GetDIBits(
            hdc_mem,
            hbm,
            0,
            height as u32,
            Some(bgra_buf.as_mut_ptr() as *mut c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );

        SelectObject(hdc_mem, old_bm);
        let _ = DeleteObject(hbm.into());
        let _ = DeleteDC(hdc_mem);
        ReleaseDC(None, hdc_screen);

        let mut rgba_buf = vec![0u8; bgra_buf.len()];
        for i in 0..(width * height) as usize {
            let b = bgra_buf[i * 4];
            let g = bgra_buf[i * 4 + 1];
            let r = bgra_buf[i * 4 + 2];
            let a = bgra_buf[i * 4 + 3];

            rgba_buf[i * 4] = r;
            rgba_buf[i * 4 + 1] = g;
            rgba_buf[i * 4 + 2] = b;
            rgba_buf[i * 4 + 3] = if a == 0 { 255 } else { a };
        }

        image::save_buffer(
            out_path,
            &rgba_buf,
            width as u32,
            height as u32,
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|e| windows::core::Error::new(windows::core::HRESULT(-1), format!("GDI Save error: {}", e)))?;

        Ok((width as u32, height as u32))
    }
}

fn ensure_parent_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let _ = fs::create_dir_all(parent);
        }
    }
}

fn main() {
    let options = match parse_cli_args() {
        Ok(opts) => opts,
        Err(err) => {
            print_agent_stderr(&format!("[CAPTURE_ERROR]\nerror: invalid_arguments\ndetail: {}", err));
            std::process::exit(1);
        }
    };

    let abs_out_path = match fs::canonicalize(&options.output_path) {
        Ok(p) => p,
        Err(_) => {
            ensure_parent_dir(&options.output_path);
            let cur = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            cur.join(&options.output_path)
        }
    };

    let console_hwnd = unsafe { GetConsoleWindow() };
    if console_hwnd != HWND::default() {
        unsafe {
            let _ = ShowWindow(console_hwnd, SW_HIDE);
        }
    }
    let _guard = ConsoleGuard(console_hwnd);

    match options.target {
        TargetMode::FullScreen => {
            let hmonitor = unsafe {
                let pt = POINT { x: 0, y: 0 };
                MonitorFromPoint(pt, MONITOR_DEFAULTTOPRIMARY)
            };

            let mut captured = false;
            let mut res_w = 0;
            let mut res_h = 0;
            let mut mode_used = "fullscreen_wgc";

            if let Ok(interop) = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>() {
                if let Ok(item) = unsafe { interop.CreateForMonitor(hmonitor) } {
                    if let Ok((w, h)) = capture_wgc(&item, &abs_out_path, options.no_border) {
                        captured = true;
                        res_w = w;
                        res_h = h;
                    }
                }
            }

            if !captured {
                if let Ok((w, h)) = capture_gdi_screen(&abs_out_path) {
                    captured = true;
                    res_w = w;
                    res_h = h;
                    mode_used = "fullscreen_gdi_fallback";
                }
            }

            if captured {
                print_agent_stdout(&format!(
                    "[CAPTURED_IMAGE]\nstatus: success\nmode: {}\nresolution: {}x{}\npath: {}",
                    mode_used,
                    res_w,
                    res_h,
                    abs_out_path.display()
                ));
                std::process::exit(0);
            } else {
                print_agent_stderr("[CAPTURE_ERROR]\nerror: fullscreen_capture_failed");
                std::process::exit(1);
            }
        }
        ref specific_target => {
            let all_windows = collect_all_windows();

            let matched = all_windows.iter().find(|w| match specific_target {
                TargetMode::ByPid(pid) => w.pid == *pid,
                TargetMode::ByProcessName(name) => {
                    let clean_name = name.trim_end_matches(".exe").to_lowercase();
                    let proc_clean = w.process_name.trim_end_matches(".exe").to_lowercase();
                    proc_clean == clean_name || proc_clean.contains(&clean_name)
                }
                TargetMode::ByTitle(title_query) => {
                    let q = title_query.to_lowercase();
                    w.title.to_lowercase().contains(&q)
                }
                TargetMode::FullScreen => false,
            }).cloned();

            let target_win = match matched {
                Some(w) => w,
                None => {
                    let mut list_out = String::from("available_windows:\n");
                    for win in all_windows.iter().take(30) {
                        list_out.push_str(&format!(
                            "pid={}|process={}|title={}\n",
                            win.pid, win.process_name, win.title
                        ));
                    }
                    print_agent_stderr(&format!(
                        "[CAPTURE_ERROR]\nerror: no_matching_window\n{}",
                        list_out.trim_end()
                    ));
                    std::process::exit(1);
                }
            };

            // Restore if minimized
            unsafe {
                if IsIconic(target_win.hwnd).as_bool() {
                    let _ = ShowWindowAsync(target_win.hwnd, SW_RESTORE);
                    std::thread::sleep(Duration::from_millis(150));
                }
            }

            let mut captured = false;
            let mut res_w = 0;
            let mut res_h = 0;
            let mut mode_used = "window_wgc";

            if let Ok(interop) = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>() {
                if let Ok(item) = unsafe { interop.CreateForWindow(target_win.hwnd) } {
                    if let Ok((w, h)) = capture_wgc(&item, &abs_out_path, options.no_border) {
                        captured = true;
                        res_w = w;
                        res_h = h;
                    }
                }
            }

            if !captured {
                if let Ok((w, h)) = capture_gdi_window(target_win.hwnd, &abs_out_path) {
                    captured = true;
                    res_w = w;
                    res_h = h;
                    mode_used = "window_gdi_fallback";
                }
            }

            if captured {
                print_agent_stdout(&format!(
                    "[CAPTURED_IMAGE]\nstatus: success\nmode: {}\ntarget_pid: {}\ntarget_process: {}\ntarget_title: {}\nresolution: {}x{}\npath: {}",
                    mode_used,
                    target_win.pid,
                    target_win.process_name,
                    target_win.title,
                    res_w,
                    res_h,
                    abs_out_path.display()
                ));
                std::process::exit(0);
            } else {
                print_agent_stderr(&format!(
                    "[CAPTURE_ERROR]\nerror: window_capture_failed\ntarget_pid: {}\ntarget_process: {}",
                    target_win.pid, target_win.process_name
                ));
                std::process::exit(1);
            }
        }
    }
}
