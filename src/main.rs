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

struct HookGuard(HHOOK);
impl Drop for HookGuard {
    fn drop(&mut self) {
        unsafe { let _ = UnhookWindowsHookEx(self.0); }
    }
}

static LAYOUTS: OnceLock<LayoutList> = OnceLock::new();
static SUPPRESS_UP: AtomicBool = AtomicBool::new(false);
static CAPS_DOWN: AtomicBool = AtomicBool::new(false);
static SHIFT_DOWN: AtomicBool = AtomicBool::new(false);

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
        let injected = (kb.flags.0 & LLKHF_INJECTED.0) != 0;
        let msg = wparam.0 as u32;
        let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
        let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

        if kb.vkCode == VK_SHIFT.0 as u32
            || kb.vkCode == VK_LSHIFT.0 as u32
            || kb.vkCode == VK_RSHIFT.0 as u32
        {
            SHIFT_DOWN.store(is_down, Ordering::Relaxed);
        }

        if kb.vkCode == VK_CAPITAL.0 as u32 && !injected {
            if is_down {
                let was_down = CAPS_DOWN.swap(true, Ordering::Relaxed);
                if !SHIFT_DOWN.load(Ordering::Relaxed) {
                    if !was_down {
                        switch_layout();
                    }
                    SUPPRESS_UP.store(true, Ordering::Relaxed);
                    return LRESULT(1);
                }
            } else if is_up {
                CAPS_DOWN.store(false, Ordering::Relaxed);
                if SUPPRESS_UP.swap(false, Ordering::Relaxed) {
                    return LRESULT(1);
                }
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

unsafe fn focused_window() -> HWND {
    let fg = GetForegroundWindow();
    if fg.0.is_null() {
        return fg;
    }
    let tid = GetWindowThreadProcessId(fg, None);
    let mut gui = GUITHREADINFO {
        cbSize: std::mem::size_of::<GUITHREADINFO>() as u32,
        ..Default::default()
    };
    if GetGUIThreadInfo(tid, &mut gui).is_ok() && !gui.hwndFocus.0.is_null() {
        gui.hwndFocus
    } else {
        fg
    }
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
    let next = layouts[(idx + 1) % layouts.len()];
    let _ = PostMessageW(
        hwnd,
        WM_INPUTLANGCHANGEREQUEST,
        WPARAM(0),
        LPARAM(next.0 as usize as isize),
    );
}

unsafe fn get_layout_list() -> Vec<HKL> {
    loop {
        let count = GetKeyboardLayoutList(None) as usize;
        let mut list = vec![HKL::default(); count];
        let filled = GetKeyboardLayoutList(Some(&mut list)) as usize;
        if filled == count {
            list.truncate(filled);
            return list;
        }
    }
}

fn main() {
    unsafe {
        let list = get_layout_list();
        let _ = LAYOUTS.set(LayoutList(list));

        let hmod = GetModuleHandleW(None).unwrap();
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), hmod, 0).unwrap();
        let _guard = HookGuard(hook);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
