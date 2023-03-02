use std::ptr::null;
use std::{thread, time};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, LoadIconW, SendMessageW, ICON_BIG, ICON_SMALL, WM_SETICON};

// winres set_icon or set_icon_with_id must be used in the build for this to work
pub fn set_window_icon(id: u16, class_name: &'static str, window_name: &'static str) {
    thread::spawn(move || {
        let mut c_name_utf16: Vec<u16> = class_name.encode_utf16().collect();
        c_name_utf16.push(0);
        let c_name_ptr = PCWSTR(c_name_utf16.as_mut_ptr());

        let mut w_name_utf16: Vec<u16> = window_name.encode_utf16().collect();
        w_name_utf16.push(0);
        let w_name_ptr = PCWSTR(w_name_utf16.as_mut_ptr());

        for _ in 0..10 {
            let hwnd = unsafe { FindWindowW(c_name_ptr, w_name_ptr) };
            if hwnd == HWND(0) {
                thread::sleep(time::Duration::from_millis(10));
                continue;
            }

            let hicon = match unsafe { LoadIconW(GetModuleHandleW(PCWSTR(null::<u16>())).unwrap(), PCWSTR(id as *const u16)) } {
                Ok(hicon) => hicon.0,
                _ => {
                    eprintln!("No Icon #{id} in resource");
                    break;
                }
            };

            unsafe { SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_SMALL as usize), LPARAM(hicon)) };
            unsafe { SendMessageW(hwnd, WM_SETICON, WPARAM(ICON_BIG as usize), LPARAM(hicon)) };
            break;
        }
    });
}
