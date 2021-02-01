use bindings::{
    windows::win32::menus_and_resources::{LoadCursorA, HMENU},
    windows::win32::system_services::{
        GetModuleHandleA, CW_USEDEFAULT, GWLP_USERDATA, HINSTANCE, IDC_ARROW, LRESULT, WM_DESTROY,
        WM_NCCREATE, WS_EX_NOREDIRECTIONBITMAP, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    },
    windows::win32::windows_and_messaging::{
        CreateWindowExA, DefWindowProcA, PostQuitMessage, RegisterClassA, CREATESTRUCTA, HWND,
        LPARAM, WNDCLASSA, WPARAM,
    },
};
use std::sync::Once;

static REGISTER_CLASS: Once = Once::new();

#[link(name = "user32")]
extern "system" {
    fn SetWindowLongPtrA(h_wnd: HWND, n_index: i32, dw_new_long: i64) -> i64;
}

#[link(name = "user32")]
extern "system" {
    fn GetWindowLongPtrA(h_wnd: HWND, n_index: i32) -> i64;
}

pub struct BasicWindow {
    window_handle: HWND,
}

impl BasicWindow {
    pub fn new(width: u32, height: u32, class_name: &str, title: &str) -> Box<Self> {
        let window = Box::new(BasicWindow {
            window_handle: HWND(0),
        });
        let window_ptr = Box::into_raw(window);
        unsafe {
            let instance = HINSTANCE(GetModuleHandleA(std::ptr::null()));
            debug_assert!(instance.0 != 0);
            let class_name: Vec<u8> = format!("{}\0", class_name).into();
            let title: Vec<u8> = format!("{}\0", title).into();

            REGISTER_CLASS.call_once(|| {
                let mut wc = WNDCLASSA::default();
                wc.h_cursor = LoadCursorA(HINSTANCE(0), IDC_ARROW as *const i8);
                wc.h_instance = instance;
                wc.lpsz_class_name = class_name.as_ptr() as *mut u8 as *mut i8;
                wc.style = 0;
                wc.lpfn_wnd_proc = Some(Self::wnd_proc);
                let atom = RegisterClassA(&wc);
                debug_assert!(atom != 0);
            });

            CreateWindowExA(
                WS_EX_NOREDIRECTIONBITMAP as u32,
                class_name.as_ptr() as *const _ as *const _,
                title.as_ptr() as *const _ as *const _,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                HWND(0),
                HMENU(0),
                instance,
                window_ptr as *mut _,
            );
            let window = Box::from_raw(window_ptr);
            assert!(window.window_handle.0 != 0);
            window
        }
    }

    pub fn handle(&self) -> HWND {
        self.window_handle
    }

    fn message_handler(&self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe {
            match message as i32 {
                WM_DESTROY => {
                    PostQuitMessage(0);
                    return LRESULT(0);
                }
                _ => {}
            }
            DefWindowProcA(self.window_handle, message, wparam, lparam)
        }
    }

    extern "system" fn wnd_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message as i32 == WM_NCCREATE {
                let create_struct: *mut CREATESTRUCTA = std::mem::transmute(lparam);
                let window_raw: *mut BasicWindow =
                    std::mem::transmute(create_struct.as_ref().unwrap().lp_create_params);
                window_raw.as_mut().unwrap().window_handle = window;
                SetWindowLongPtrA(window, GWLP_USERDATA, window_raw as _);
            } else {
                let window_raw: *mut BasicWindow =
                    std::mem::transmute(GetWindowLongPtrA(window, GWLP_USERDATA));
                if let Some(window) = window_raw.as_ref() {
                    return window.message_handler(message, wparam, lparam);
                }
            }

            DefWindowProcA(window, message, wparam, lparam)
        }
    }
}
