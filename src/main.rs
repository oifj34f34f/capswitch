#![windows_subsystem = "windows"]

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use windows::Win32::Foundation::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;

struct LayoutList(Vec<HKL>);
unsafe impl Sync for LayoutList {}
unsafe impl Send for LayoutList {}

static LAYOUTS: OnceLock<LayoutList> = OnceLock::new();
static SUPPRESS_UP: AtomicBool = AtomicBool::new(false);
static CAPS_DOWN: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        if kb.vkCode == VK_CAPITAL.0 as u32 {
            let injected = (kb.flags.0 & 0x10) != 0;
            if !injected {
                let msg = wparam.0 as u32;
                if msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN {
                    let was_down = CAPS_DOWN.swap(true, Ordering::SeqCst);
                    let shift = (GetAsyncKeyState(VK_SHIFT.0 as i32) as u16 & 0x8000) != 0;
                    if !shift {
                        if !was_down {
                            switch_layout();
                        }
                        SUPPRESS_UP.store(true, Ordering::SeqCst);
                        return LRESULT(1);
                    }
                } else if msg == WM_KEYUP || msg == WM_SYSKEYUP {
                    CAPS_DOWN.store(false, Ordering::SeqCst);
                    if SUPPRESS_UP.swap(false, Ordering::SeqCst) {
                        return LRESULT(1);
                    }
                }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

unsafe fn focused_window() -> HWND {
    let fg = GetForegroundWindow();
    if !fg.0.is_null() {
        let tid = GetWindowThreadProcessId(fg, None);
        let mut gui = GUITHREADINFO {
            cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
            ..Default::default()
        };
        if GetGUIThreadInfo(tid, &mut gui).is_ok() && !gui.hwndFocus.0.is_null() {
            return gui.hwndFocus;
        }
    }
    fg
}

unsafe fn switch_layout() {
    let layouts = match LAYOUTS.get() {
        Some(l) => &l.0,
        None => return,
    };
    if layouts.len() < 2 {
        return;
    }
    let hwnd = focused_window();
    if hwnd.0.is_null() {
        return;
    }
    let tid = GetWindowThreadProcessId(hwnd, None);
    let current = GetKeyboardLayout(tid);
    let idx = layouts.iter().position(|h| h.0 == current.0).unwrap_or(0);
    let next_idx = (idx + 1) % layouts.len();
    let next = layouts[next_idx].0 as isize;
    let _ = PostMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, WPARAM(0), LPARAM(next));
}

fn main() {
    unsafe {
        let count = GetKeyboardLayoutList(None) as usize;
        let mut list = vec![HKL::default(); count];
        GetKeyboardLayoutList(Some(&mut list));
        let _ = LAYOUTS.set(LayoutList(list));

        let hmod = GetModuleHandleW(None).unwrap();
        let hinst: HINSTANCE = hmod.into();
        let _hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hinst, 0).unwrap();

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
