# COSMIC Tiling Exceptions

COSMIC Tiling Exceptions is a GUI utility built with `libcosmic` to manage window auto-tiling exceptions on the COSMIC Desktop Environment (Pop!_OS). It allows you to easily bypass the Wayland auto-tiling mechanism for specific applications, making them open in floating mode automatically.

## Requirements

This application natively integrates with the COSMIC Compositor to fetch open windows. It requires the `cosmic-ext-window-helper` script to function properly.

### 1. Install `cosmic-ext-window-helper`
You must install the helper script via `pipx` before using this application:
```bash
sudo apt update
sudo apt install pipx
pipx install cosmic-ext-window-helper
pipx ensurepath
```

## Installation

You can download the `.deb`, `.rpm`, or `.AppImage` from the Releases page.

### Debian / Ubuntu / Pop!_OS
```bash
sudo apt install ./cosmic-tiling-manager_*.deb
```

### Fedora
```bash
sudo dnf install ./cosmic-tiling-manager_*.rpm
```

## Building from source

```bash
cargo build --release
```
