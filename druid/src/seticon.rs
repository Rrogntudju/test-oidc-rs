use std::{thread, time};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winnt::LPCWSTR;
use winapi::{
    shared::minwindef::{LPARAM, WPARAM},
    shared::windef::{HICON, HWND},
    um::winuser::{FindWindowW, LoadIconW, SendMessageW, ICON_BIG, ICON_SMALL, MAKEINTRESOURCEW, WM_SETICON},
};

pub fn set_window_icon(class_name: &'static str, window_name: &'static str) {
    thread::spawn(move || {
        let mut c_name_utf16: Vec<u16> = class_name.encode_utf16().collect();
        c_name_utf16.push(0);
        let c_name_ptr = c_name_utf16.as_mut_ptr() as LPCWSTR;

        let mut w_name_utf16: Vec<u16> = window_name.encode_utf16().collect();
        w_name_utf16.push(0);
        let w_name_ptr = w_name_utf16.as_mut_ptr() as LPCWSTR;

        for _ in 0..10 {
            let hwnd = unsafe { FindWindowW(c_name_ptr, w_name_ptr) };
            if hwnd == 0 as HWND {
                thread::sleep(time::Duration::from_millis(10));
                continue;
            }

            let hicon = unsafe { LoadIconW(GetModuleHandleW(0 as LPCWSTR), MAKEINTRESOURCEW(1)) };
            if hicon == 0 as HICON {
                panic!("No Icon #1 in resource");
            }

            unsafe { SendMessageW(hwnd, WM_SETICON, ICON_SMALL as WPARAM, hicon as LPARAM) };
            unsafe { SendMessageW(hwnd, WM_SETICON, ICON_BIG as WPARAM, hicon as LPARAM) };
        }
    });
}
