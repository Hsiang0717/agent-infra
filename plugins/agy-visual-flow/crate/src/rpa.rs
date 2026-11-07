use windows::core::s;
use windows::Win32::Foundation::{GetLastError, HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::StationsAndDesktops::{
    OpenDesktopA, OpenWindowStationA, SetProcessWindowStation, SetThreadDesktop,
    DF_ALLOWOTHERACCOUNTHOOK,
};
use windows::Win32::System::Threading::{AttachThreadInput, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, INPUT_MOUSE, KEYBDINPUT, KEYEVENTF_KEYUP,
    KEYEVENTF_UNICODE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEINPUT, VIRTUAL_KEY,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, GetWindowThreadProcessId, SetCursorPos, SetForegroundWindow,
    ShowWindow, SwitchToThisWindow, SM_CXSCREEN, SM_CYSCREEN, SW_RESTORE,
};

/// Ensure current thread and process are attached to interactive desktop (WinSta0\Default)
pub fn ensure_interactive_desktop() {
    unsafe {
        if let Ok(winsta0) = OpenWindowStationA(s!("WinSta0"), false, 0x10000000) {
            let _ = SetProcessWindowStation(winsta0);
        }
        if let Ok(desk) = OpenDesktopA(s!("Default"), DF_ALLOWOTHERACCOUNTHOOK, false, 0x10000000) {
            let _ = SetThreadDesktop(desk);
        }
    }
}

/// Deterministic Click Execution:
/// 1. Switch station/desktop to WinSta0\Default (bypassing service/CLI desktop isolation).
/// 2. Bring target window to foreground & attach thread input (if hwnd provided).
/// 3. Direct cursor jump to exact target coordinates (no slow interpolation).
/// 4. Zero-delay full event-pair trigger (DOWN followed by UP).
pub fn execute_deterministic_click(target_x: i32, target_y: i32, hwnd: Option<HWND>) -> Result<(), String> {
    ensure_interactive_desktop();

    unsafe {
        let cur_tid = GetCurrentThreadId();
        let mut attached_tid = None;

        // 1. Focus & input queue attachment
        if let Some(h) = hwnd {
            let mut target_pid = 0;
            let target_tid = GetWindowThreadProcessId(h, Some(&mut target_pid));
            if target_tid != 0 && target_tid != cur_tid {
                let _ = AttachThreadInput(cur_tid, target_tid, true);
                attached_tid = Some(target_tid);
            }
            let _ = ShowWindow(h, SW_RESTORE);
            let _ = SetForegroundWindow(h);
            SwitchToThisWindow(h, true);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // 2. Direct cursor placement
        if SetCursorPos(target_x, target_y).is_err() {
            if let Some(tid) = attached_tid {
                let _ = AttachThreadInput(cur_tid, tid, false);
            }
            return Err(format!("Failed to set cursor position to ({}, {}), err={:?}", target_x, target_y, GetLastError()));
        }

        // 3. Compute normalized absolute coordinates for SendInput (0..65535)
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        if screen_w <= 0 || screen_h <= 0 {
            if let Some(tid) = attached_tid {
                let _ = AttachThreadInput(cur_tid, tid, false);
            }
            return Err("Failed to query screen metrics".to_string());
        }

        let abs_x = ((target_x as f64 / screen_w as f64) * 65535.0).round() as i32;
        let abs_y = ((target_y as f64 / screen_h as f64) * 65535.0).round() as i32;

        let inputs = [
            // Move event to update hover state
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: abs_x,
                        dy: abs_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // Left Down
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: abs_x,
                        dy: abs_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_LEFTDOWN | MOUSEEVENTF_ABSOLUTE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
            // Left Up
            INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx: abs_x,
                        dy: abs_y,
                        mouseData: 0,
                        dwFlags: MOUSEEVENTF_LEFTUP | MOUSEEVENTF_ABSOLUTE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            },
        ];

        let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);

        if let Some(tid) = attached_tid {
            let _ = AttachThreadInput(cur_tid, tid, false);
        }

        if sent != inputs.len() as u32 {
            return Err(format!("SendInput failed: sent {} of {} events, err={:?}", sent, inputs.len(), GetLastError()));
        }

        Ok(())
    }
}

/// Deterministic Text Typing Execution:
/// 1. Switch station/desktop to WinSta0\Default.
/// 2. Bring target window to foreground & attach thread input (if hwnd provided).
/// 3. Zero-delay unicode character key sequence trigger via SendInput.
pub fn execute_deterministic_type(text: &str, hwnd: Option<HWND>) -> Result<(), String> {
    ensure_interactive_desktop();

    unsafe {
        let cur_tid = GetCurrentThreadId();
        let mut attached_tid = None;

        // 1. Focus & input queue attachment
        if let Some(h) = hwnd {
            let mut target_pid = 0;
            let target_tid = GetWindowThreadProcessId(h, Some(&mut target_pid));
            if target_tid != 0 && target_tid != cur_tid {
                let _ = AttachThreadInput(cur_tid, target_tid, true);
                attached_tid = Some(target_tid);
            }
            let _ = ShowWindow(h, SW_RESTORE);
            let _ = SetForegroundWindow(h);
            SwitchToThisWindow(h, true);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        for ch in text.encode_utf16() {
            let inputs = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: ch,
                            dwFlags: KEYEVENTF_UNICODE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VIRTUAL_KEY(0),
                            wScan: ch,
                            dwFlags: KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];

            let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
            if sent != inputs.len() as u32 {
                if let Some(tid) = attached_tid {
                    let _ = AttachThreadInput(cur_tid, tid, false);
                }
                return Err(format!("SendInput failed for char {}: sent {} of {}", ch, sent, inputs.len()));
            }
            std::thread::sleep(std::time::Duration::from_millis(15));
        }

        if let Some(tid) = attached_tid {
            let _ = AttachThreadInput(cur_tid, tid, false);
        }

        Ok(())
    }
}

/// Convert client relative coordinates to physical screen coordinates
pub fn client_to_screen_coords(hwnd: HWND, client_x: i32, client_y: i32) -> Result<(i32, i32), String> {
    unsafe {
        let mut pt = POINT {
            x: client_x,
            y: client_y,
        };
        if ClientToScreen(hwnd, &mut pt).as_bool() {
            Ok((pt.x, pt.y))
        } else {
            Err("ClientToScreen failed".to_string())
        }
    }
}

/// Get client dimensions of a window
pub fn get_window_client_dimensions(hwnd: HWND) -> Result<(u32, u32), String> {
    unsafe {
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect).is_ok() {
            let w = (rect.right - rect.left).max(0) as u32;
            let h = (rect.bottom - rect.top).max(0) as u32;
            Ok((w, h))
        } else {
            Err("GetClientRect failed".to_string())
        }
    }
}
