# cava-wallpaper

A simple installer and launcher for https://github.com/rs-pro0/wallpaper-cava — a real-time music visualizer that renders system audio as a live wallpaper using CAVA and Rust.

This repository makes it easy to install and use `wallpaper-cava` globally as the `cava-wallpaper` command.

---

## Requirements

- Git
- A Unix-like environment (Linux)
- ALSA/PulseAudio or another audio backend supported by CAVA
- Optional: Rust toolchain or build tools if you want to build from source

See the installer script for exact dependency and platform details.

## Installation

Clone this repository and run the installer:

```bash
git clone https://github.com/viztini/cava-wallpaper.git
cd cava-wallpaper
chmod +x install.sh
./install.sh
```

The installer will install or make `cava-wallpaper` available system-wide. Review `install.sh` for what it will do and any prompts it shows.

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

- CAVA configuration is typically located at `~/.config/cava/config` — adjust it for your audio source and visual preferences.
- `wallpaper-cava` handles setting the wallpaper; compositor and desktop environment behavior may vary (X11 vs Wayland).

## Uninstall

Remove the installed binary and any installed files (adjust paths according to your system and what `install.sh` did):

```bash
sudo rm /usr/local/bin/cava-wallpaper
# remove other files if the installer placed them elsewhere
```

Inspect `install.sh` for exact installed paths and reverse those steps to fully uninstall.

## Troubleshooting

- If you don't see the visualizer, confirm CAVA works standalone and that audio capture is functional.
- Run `cava-wallpaper` from a terminal to view stderr/stdout for errors.
- If installation fails due to permissions, re-run the installer with appropriate privileges or move files manually.

## Contributing

Bug reports, issues and pull requests are welcome. If you'd like me to open a PR with further README edits, tell me what to include.

## License

Add your project's license here or include a LICENSE file in the repository.

## Acknowledgements

- Upstream: https://github.com/rs-pro0/wallpaper-cava
