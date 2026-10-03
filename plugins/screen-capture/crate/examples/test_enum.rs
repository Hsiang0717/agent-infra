use windows::core::{s, BOOL, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT, TRUE};
use windows::Win32::System::StationsAndDesktops::{CloseDesktop, EnumDesktopWindows, OpenDesktopA, SetThreadDesktop, DESKTOP_CONTROL_FLAGS};
use windows::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible};
use std::path::Path;

fn main() {
    unsafe {
        let desk = OpenDesktopA(s!("Default"), DESKTOP_CONTROL_FLAGS(0), false, 0x01FF);
        if let Ok(hdesk) = desk {
            let _ = SetThreadDesktop(hdesk);
            
            unsafe extern "system" fn callback(hwnd: HWND, _: LPARAM) -> BOOL {
                unsafe {
                    if !IsWindowVisible(hwnd).as_bool() {
                        return TRUE;
                    }
                    let mut rect = RECT::default();
                    if GetWindowRect(hwnd, &mut rect).is_err() {
                        return TRUE;
                    }
                    if rect.right - rect.left < 50 || rect.bottom - rect.top < 50 {
                        return TRUE;
                    }
                    let mut pid = 0u32;
                    GetWindowThreadProcessId(hwnd, Some(&mut pid));
                    let mut buf = [0u16; 512];
                    let len = GetWindowTextW(hwnd, &mut buf);
                    let title = String::from_utf16_lossy(&buf[..len as usize]).trim().to_string();
                    if title.is_empty() || title == "Default IME" || title == "MSCTFIME UI" || title == "Program Manager" {
                        return TRUE;
                    }

                    let mut proc_name = String::from("unknown");
                    if let Ok(h) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                        let mut path_buf = [0u16; 1024];
                        let mut path_len = path_buf.len() as u32;
                        if QueryFullProcessImageNameW(h, PROCESS_NAME_FORMAT(0), PWSTR(path_buf.as_mut_ptr()), &mut path_len).is_ok() && path_len > 0 {
                            let full = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                            if let Some(f) = Path::new(&full).file_name() {
                                proc_name = f.to_string_lossy().to_string();
                            }
                        }
                        let _ = CloseHandle(h);
                    }

                    println!("pid={}|process={}|title={}", pid, proc_name, title);
                    TRUE
                }
            }

            let _ = EnumDesktopWindows(Some(hdesk), Some(callback), LPARAM(0));
            let _ = CloseDesktop(hdesk);
        }
    }
}
