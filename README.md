# cava-wallpaper

A simple installer and launcher for [wallpaper-cava](https://github.com/rs-pro0/wallpaper-cava) — a real-time music visualizer that renders system audio as a live wallpaper using CAVA and Rust.

This repository makes it easy to install and use `wallpaper-cava` globally as the `cava-wallpaper` command.

---

## Requirements

- **Git** - for cloning repositories
- **Rust toolchain** - `cargo` is required to build and install
- **Linux** with Wayland compositor support
- **ALSA/PulseAudio** or another audio backend supported by CAVA

Install Rust from [rustup.rs](https://rustup.rs/) if you don't have it already.

## Installation

Clone this repository and run the installer:

```bash
git clone https://github.com/viztini/cava-wallpaper.git
cd cava-wallpaper
./install.sh
```

The installer will:
1. Clone the upstream `wallpaper-cava` repository to `~/cava-wallpaper-src`
2. Build and install the binary as `cava-wallpaper` using `cargo install`
3. Copy the config file to `~/.config/cava-wallpaper/config.toml`
4. Add `~/.cargo/bin` to your PATH if needed

After installation, you may need to restart your shell or run `source ~/.bashrc`.

## Usage

Run the command after installation:

```bash
cava-wallpaper
```

To run it in the background:

```bash
cava-wallpaper & disown
```

Refer to the upstream `wallpaper-cava` project for configuration options and platform-specific behavior.

## Configuration

Configuration file is located at `~/.config/cava-wallpaper/config.toml`. Edit this file to customize:
- Audio source and device
- Visual appearance and colors
- Bar settings and behavior

You can also pass a custom config path:
```bash
cava-wallpaper --config /path/to/your/config.toml
```

**Note:** CAVA configuration at `~/.config/cava/config` is separate and may affect audio input behavior.

## Uninstall

To completely remove cava-wallpaper:

```bash
# Remove the installed binary
cargo uninstall wallpaper-cava

# Remove source directory
rm -rf ~/cava-wallpaper-src

# Remove config directory
rm -rf ~/.config/cava-wallpaper
```

## Troubleshooting

- **Visualizer not appearing?** Ensure your compositor supports the wlr-layer-shell protocol (most Wayland compositors do)
- **No audio visualization?** Check that audio is playing and CAVA can capture it. Test with standalone CAVA first.
- **Command not found?** Make sure `~/.cargo/bin` is in your PATH. Run `source ~/.bashrc` or restart your terminal.
- **Build errors?** Ensure you have the latest Rust toolchain: `rustup update`

For detailed errors, run `cava-wallpaper` from a terminal to see output.

## Contributing

Bug reports, issues, and pull requests are welcome!

## License

MIT License

## Acknowledgements

- Upstream project: [rs-pro0/wallpaper-cava](https://github.com/rs-pro0/wallpaper-cava)
