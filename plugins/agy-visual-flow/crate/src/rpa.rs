use windows::Win32::Foundation::{HWND, POINT, RECT};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClientRect, GetSystemMetrics, SetCursorPos, SetForegroundWindow, SM_CXSCREEN, SM_CYSCREEN,
};

/// Deterministic Click Execution:
/// 1. Bring target window to foreground (if hwnd provided).
/// 2. Direct cursor jump to exact target coordinates (no slow interpolation).
/// 3. Zero-delay full event-pair trigger (DOWN followed by UP).
pub fn execute_deterministic_click(target_x: i32, target_y: i32, hwnd: Option<HWND>) -> Result<(), String> {
    unsafe {
        // 1. Focus assurance
        if let Some(h) = hwnd {
            let _ = SetForegroundWindow(h);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // 2. Direct cursor placement
        if SetCursorPos(target_x, target_y).is_err() {
            return Err(format!("Failed to set cursor position to ({}, {})", target_x, target_y));
        }

        // 3. Compute normalized absolute coordinates for SendInput (0..65535)
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        if screen_w <= 0 || screen_h <= 0 {
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
        if sent != inputs.len() as u32 {
            return Err(format!("SendInput failed: sent {} of {} events", sent, inputs.len()));
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
