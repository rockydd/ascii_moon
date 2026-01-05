# ASCII Moon

A simple and beautiful Terminal User Interface (TUI) to show the current phase of the moon in ASCII art.

[![asciicast](https://asciinema.org/a/6hv6n1VqamlmJYLZA68gdQNhc.svg)](https://asciinema.org/a/6hv6n1VqamlmJYLZA68gdQNhc)

This project is heavily inspired by the beautiful website [asciimoon.com](https://asciimoon.com/).

---

## Features

*   **Real-time Moon Phase:** Displays an ASCII art representation of the moon's phase for any given date.
*   **Interactive TUI:**
    *   Use the **Left** and **Right** arrow keys to travel through time day-by-day.
    *   Toggle labels for major lunar features with the **'l'** key.
    *   Cycle through multiple languages for labels (English, Chinese, French, Japanese, and Spanish) with the **'L'** key.
    *   Toggle hiding the unlit (dark) part of the moon with the **'d'** key.
    *   Toggle a **Moon Poem** panel (matching the current language) with the **'p'** key, and pick a new random poem with **'P'**.
    *   Toggle the information panel with the **'i'** key.
*   **Cross-Platform:** Works on Linux, macOS, and Windows.
*   **Lightweight:** It's a single, small, native binary.
*   **Non-Interactive Mode:** Print the moon to the console at a specified size for use in scripts or MOTD.

## Installation

### Homebrew (macOS)

1.  Tap the repository:
    ```sh
    brew tap rockydd/tap
    ```
2.  Install the formula:
    ```sh
    brew install ascii_moon
    ```

### From Releases

You can download the latest pre-compiled binaries for Linux, macOS, and Windows from the [GitHub Releases](https://github.com/rockydd/ascii_moon/releases) page.

### From Source

If you have Rust installed, you can build it from source:

```sh
git clone https://github.com/rockydd/ascii_moon.git
cd ascii_moon
cargo build --release
./target/release/ascii_moon
```

## Usage

## Custom Poems

`ascii_moon` can load moon poems from a folder on disk, so you can add your own poems easily.

- **Default**: looks for `./poems/` (relative to where you run the command). If not found, it also tries:
  - `<prefix>/share/ascii_moon/poems` (Homebrew-style install location)
- **Override**: pass `--poems-dir /path/to/poems`

### Folder layout

Create subfolders by language:

- `poems/en/`
- `poems/zh/`
- `poems/fr/`
- `poems/ja/`
- `poems/es/`

### File format (`.txt`)

Each poem file is simple:

- Line 1: **Title**
- Line 2: **Author**
- Line 3: `---`
- Remaining lines: poem body

## Theme (white terminal background)

If your terminal uses a **light/white** background, the default “soft pastel” poem colors may look too faint.
Use:

- `--theme light` for higher-contrast poem colors on white backgrounds
- `--theme dark` for dark terminals
- `--theme auto` (default) tries to infer from `COLORFGBG` when available

### Interactive Mode

Run the application without arguments to launch the full-screen interactive TUI:

```sh
ascii_moon
```

To see the moon on a specific date in interactive mode:

```sh
ascii_moon --date YYYY-MM-DD
```

By default (when `--date` is not provided), the TUI follows the current time and **auto-refreshes every 5 minutes**.
You can change the refresh period (or disable it) with:

```sh
ascii_moon --refresh-minutes 5
ascii_moon --refresh-minutes 0
```

### Non-Interactive (Print) Mode

For scripting or MOTD (Message of the Day) use, you can print the moon directly to the console. Use the `--lines` flag to specify the height of the output.

```sh
ascii_moon --lines 20
```

To hide the unlit (dark) part in print mode:

```sh
ascii_moon --lines 20 --hide-dark
```

### Controls (Interactive Mode)

*   **←**: Go back one day.
*   **→**: Go forward one day.
*   **n**: Switch back to **Now (auto)** mode (follows current time and auto-refreshes).
*   **l**: Toggle labels for lunar features.
*   **L**: Cycle through languages for the labels.
*   **d**: Toggle hiding the unlit (dark) part of the moon.
*   **p**: Toggle the Moon Poem panel (animated, language-matched).
*   **P**: Pick a new random poem (current language).
*   **i**: Toggle the information panel.
*   **q** or **<Esc>**: Quit the application.
*   Note: Using **←/→** switches the app into **Manual** mode (stops following "Now").

## License

This project is licensed under the MIT License.
