# caps_switcher

Switch keyboard layouts using the <kbd>Caps Lock</kbd>, and use the standard Caps Lock function by pressing <kbd>Shift</kbd>+<kbd>Caps Lock</kbd>.

Supports Windows only.

## Quick Start

Downloads, adds to startup, and launches automatically:

```powershell
irm https://github.com/oifj34f34f/caps_switcher/releases/download/latest/caps_switcher.exe -OutFile "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\caps_switcher.exe"; Start-Process "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\caps_switcher.exe"
```

<details>
<summary>Install manually</summary>

Download the binary from the [releases page](https://github.com/oifj34f34f/caps_switcher/releases/latest), then press <kbd>Win</kbd>+<kbd>R</kbd>, type `shell:startup`, and copy the binary or its shortcut to that folder.

</details>

## Uninstall

```powershell
Stop-Process -Name caps_switcher -Force -ErrorAction SilentlyContinue; Remove-Item "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\caps_switcher.exe"
```

---

*Written by AI.*
