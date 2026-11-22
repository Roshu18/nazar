<div align="center">

<img src="assets/logo.png" width="112" alt="Nazar logo">

# Nazar

### A file manager for programs.

See every place a Windows program lives, and remove it completely.

![Windows](https://img.shields.io/badge/Windows-10%20%7C%2011-0078D6?style=for-the-badge&logo=windows&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-2024-B7410E?style=for-the-badge&logo=rust&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)

**[Download the latest release](https://github.com/Roshu18/nazar/releases/latest)**

</div>

## <img src="assets/icons/about.svg" width="24" align="top"> What is Nazar?

Nazar lists every program installed on Windows. Select one and it shows every place that program lives: install folder, data folders, shortcuts, registry keys, autostart entries and services. Tick what you no longer want and delete it.

Nazar can also watch a program while it runs and record exactly what it changes, including hidden files and changes made by helper processes, without a kernel driver.

## <img src="assets/icons/features.svg" width="24" align="top"> Features

- <img src="assets/icons/dot.svg" width="10"> **Installed programs:** a searchable, sortable list with each program's icon and size
- <img src="assets/icons/dot.svg" width="10"> **Locations:** every folder, shortcut and registry entry of a program, with the reason it is listed and how certain the match is
- <img src="assets/icons/dot.svg" width="10"> **Uninstall:** runs the program's own uninstaller, then shows what it left behind
- <img src="assets/icons/dot.svg" width="10"> **Leftovers:** finds folders and registry entries from programs removed earlier
- <img src="assets/icons/dot.svg" width="10"> **Safe delete:** files go to the Recycle Bin and registry entries are backed up to a `.reg` file first
- <img src="assets/icons/dot.svg" width="10"> **Track a program:** run an installer, app or script under Nazar and get the exact list of what it created, changed or deleted

## <img src="assets/icons/shield.svg" width="24" align="top"> Built to be safe

- Windows system folders and keys shared by other programs are refused.
- Nothing is deleted permanently: files go to the Recycle Bin, registry entries are exported first.
- The app itself never runs as administrator. It asks for permission only when it needs to delete or track.

## <img src="assets/icons/download.svg" width="24" align="top"> Install

1. Download the zip from the [latest release](https://github.com/Roshu18/nazar/releases/latest).
2. Extract it to any folder. Keep `nazar-desktop.exe` and `nazar.exe` together.
3. Run `nazar-desktop.exe`.

## <img src="assets/icons/terminal.svg" width="24" align="top"> Command line

The engine `nazar.exe` also works on its own from an administrator terminal:

```powershell
nazar programs                          # list installed programs
nazar locate winrar                     # every location of a program
nazar leftovers                         # what removed programs left behind
nazar watch C:\Downloads\setup.exe      # record everything a program changes
nazar list                              # recorded traces
nazar show <trace> --removal-plan       # what a trace changed, and how to undo it
```

## <img src="assets/icons/build.svg" width="24" align="top"> Build from source

Needs Windows 10 or 11, [Rust](https://rustup.rs) 1.85+ and [Node.js](https://nodejs.org) 22+.

```powershell
git clone https://github.com/Roshu18/nazar.git
cd nazar\desktop
npm install
npx tauri build --no-bundle
cd ..
cargo build --release -p nazar-cli
.\target\release\nazar-desktop.exe
```

Run the checks:

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets
cargo test --workspace
cd desktop; npm run check
```

## <img src="assets/icons/layers.svg" width="24" align="top"> Project layout

| Part | Purpose |
|---|---|
| `crates/nazar-core` | Pure logic: attribution, net-effect folding, manifests, removal plans |
| `crates/nazar-windows` | Windows adapters: ETW, registry, volumes, elevation |
| `crates/nazar-store` | Where recorded traces are kept |
| `crates/nazar-cli` | The `nazar` engine, runs elevated |
| `desktop` | The desktop app, built with Tauri and Svelte |

## <img src="assets/icons/alert.svg" width="24" align="top"> Status

- Nazar is **early software** (0.1).
- Review what you select before deleting anything important.

<div align="center">

**If Nazar helps you keep Windows clean, please give it a star.**

</div>
