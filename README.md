# Wrain
Wrain is a rain on your wayland wallpapers. GPU accelerated via Iced. Has basic and thunderstorm modes, rain/thunder sound and wind emulation
### Basic
![basic rain](https://github.com/happyzxzxz/wrain/blob/main/gifs/basic.gif?raw=true)
### Thunderstorm
![thunderstorm rain](https://github.com/happyzxzxz/wrain/blob/main/gifs/thunder.gif?raw=true)

## Installation
### NixOS

Live Preview:
```bash
nix run github:happyzxzxz/wrain -- --mode thunderstorm
```
Install:
```bash
nix profile install github:happyzxzxz/wrain
```
Or use flakes.

To avoid a 10-minute compilation, use my Cachix cache:

1. **Install Cachix CLI**: `nix profile install nixpkgs#cachix`
2. **Trust the Wrain cache**: `cachix use wrain`
3. **Run**: `nix profile install github:happyzxzxz/wrain`

### Arch (AUR)
```bash
yay -S wrain-bin
```
Or if you want to compile it:
```bash
yay -S wrain-git
```

### Other (cargo build)
1. Install rust
2. You must install the development headers for Wayland, Vulkan, and ALSA:
#### Ubuntu/Debian:
`sudo apt install pkg-config libwayland-dev libxkbcommon-dev libasound2-dev libvulkan-dev`

#### Fedora:
`sudo dnf install pkgconf-pkg-config wayland-devel libxkbcommon-devel alsa-lib-devel vulkan-loader-devel`

#### Arch:
`sudo pacman -S base-devel wayland libxkbcommon vulkan-icd-loader alsa-lib`

Then:
```bash
git clone https://github.com/happyzxzxz/wrain.git
cd wrain
cargo build --release
```

## Usage

```bash
wrain
```
### Options
```bash
--mode MODE (basic or thunderstorm, default basic)
--no-thunder (disables thunder sound in thunderstorm mode)
--no-lightning (disables lightning in thunderstorm mode)
--no-sound (disables sound)
--rain-density DENSITY (default 700)
--rain-speed SPEED (default 1)
--rain-opacity OPACITY (default 0.3)
--volume VOLUME (if sound is on, default 0.3)
--asset-path PATH (for non nix install)
```
## Disclamer
Apologies if anything is broken, arch build is tested only in container and the code is not really good

## Contributing

Pull requests are welcome. For major changes, please open an issue first
to discuss what you would like to change.
