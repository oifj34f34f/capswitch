#define WIN32_LEAN_AND_MEAN
#include <windows.h>

static HHOOK g_hook       = NULL;
static BOOL  g_caps_down  = FALSE;
static BOOL  g_suppress   = FALSE;

static HWND focused_window(void)
{
    HWND fg = GetForegroundWindow();
    if (!fg) return fg;

    DWORD tid = GetWindowThreadProcessId(fg, NULL);

    GUITHREADINFO gti;
    gti.cbSize = sizeof(gti);

    if (GetGUIThreadInfo(tid, &gti) && gti.hwndFocus)
        return gti.hwndFocus;
    return fg;
}

static void switch_layout(void)
{
    int count = GetKeyboardLayoutList(0, NULL);
    if (count < 2) return;

    HKL *layouts = (HKL *)HeapAlloc(GetProcessHeap(), 0, count * sizeof(HKL));
    if (!layouts) return;

    int actual = GetKeyboardLayoutList(count, layouts);

    HWND hwnd = focused_window();
    if (!hwnd) {
        HeapFree(GetProcessHeap(), 0, layouts);
        return;
    }

    DWORD tid   = GetWindowThreadProcessId(hwnd, NULL);
    HKL current = GetKeyboardLayout(tid);

    int idx = 0;
    for (int i = 0; i < actual; i++) {
        if (layouts[i] == current) { idx = i; break; }
    }

    HKL next = layouts[(idx + 1) % actual];
    HeapFree(GetProcessHeap(), 0, layouts);

    PostMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, 0, (LPARAM)next);
}

static LRESULT CALLBACK hook_proc(int code, WPARAM wparam, LPARAM lparam)
{
    if (code >= 0) {
        KBDLLHOOKSTRUCT *kb = (KBDLLHOOKSTRUCT *)lparam;
        BOOL injected = (kb->flags & LLKHF_INJECTED) != 0;
        BOOL is_down  = (wparam == WM_KEYDOWN  || wparam == WM_SYSKEYDOWN);
        BOOL is_up    = (wparam == WM_KEYUP    || wparam == WM_SYSKEYUP);

        if (kb->vkCode == VK_CAPITAL && !injected) {
            if (is_down) {
                BOOL was_down = g_caps_down;
                g_caps_down = TRUE;

                if (!(GetAsyncKeyState(VK_SHIFT) & 0x8000)) {
                    if (!was_down) switch_layout();
                    g_suppress = TRUE;
                    return 1;
                }
            } else if (is_up) {
                g_caps_down = FALSE;
                if (g_suppress) {
                    g_suppress = FALSE;
                    return 1;
                }
            }
        }
    }
    return CallNextHookEx(g_hook, code, wparam, lparam);
}

void __declspec(noreturn) __stdcall RawEntryPoint(void)
{
    HINSTANCE hInstance = GetModuleHandleW(NULL);
    g_hook = SetWindowsHookExW(WH_KEYBOARD_LL, hook_proc, hInstance, 0);

    if (g_hook) {
        SetProcessWorkingSetSize(GetCurrentProcess(), (SIZE_T)-1, (SIZE_T)-1);

        MSG msg;
        while (GetMessageW(&msg, NULL, 0, 0)) {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        UnhookWindowsHookEx(g_hook);
    }

    ExitProcess(0);
}
