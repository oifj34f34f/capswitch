# capswitch

Switch keyboard layouts using the <kbd>Caps Lock</kbd>, and use the standard Caps Lock function by pressing <kbd>Shift</kbd>+<kbd>Caps Lock</kbd>.
Supports Windows only.

## Quick Start

Downloads, adds to startup, and launches automatically:

```powershell
iwr https://github.com/oifj34f34f/capswitch/releases/latest/download/capswitch.exe -OutFile "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\capswitch.exe"; Start-Process "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\capswitch.exe"
```

<details>
<summary>Install manually</summary>

Download the binary from the [releases page](https://github.com/oifj34f34f/capswitch/releases/latest), then press <kbd>Win</kbd>+<kbd>R</kbd>, type `shell:startup`, and copy the binary or its shortcut to that folder.

</details>

## Build

Requires Visual Studio Build Tools with MSVC and Windows 11 SDK. Install via winget:

```powershell
winget install -e --id Microsoft.VisualStudio.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.28000"
```

Then run in Developer PowerShell:

```powershell
cl /O2 /W4 capswitch.c /link /SUBSYSTEM:WINDOWS user32.lib kernel32.lib
```

## Uninstall

```powershell
Stop-Process -Name capswitch -Force -ErrorAction SilentlyContinue; Remove-Item "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\Startup\capswitch.exe" -Force -ErrorAction SilentlyContinue
```

---

*Written by AI.*
