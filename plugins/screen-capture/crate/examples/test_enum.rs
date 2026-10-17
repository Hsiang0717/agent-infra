use windows::Win32::Foundation::HGLOBAL;
use windows::Win32::System::DataExchange::{
    CloseClipboard, GetClipboardData, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW,
};
use windows::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

fn main() {
    unsafe {
        let seq = GetClipboardSequenceNumber();
        println!("Clipboard seq: {}", seq);
        if OpenClipboard(None).is_ok() {
            let cf_png = RegisterClipboardFormatW(windows::core::w!("PNG"));
            println!("cf_png: {}", cf_png);
            if cf_png != 0 && IsClipboardFormatAvailable(cf_png).is_ok() {
                if let Ok(handle) = GetClipboardData(cf_png) {
                    let hglobal = HGLOBAL(handle.0);
                    let ptr = GlobalLock(hglobal);
                    let size = GlobalSize(hglobal);
                    println!("PNG found in clipboard! size: {}", size);
                    let _ = GlobalUnlock(hglobal);
                }
            }
            let _ = CloseClipboard();
        }
    }
}
