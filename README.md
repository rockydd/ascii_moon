# ASCII Moon

A simple and beautiful Terminal User Interface (TUI) to show the current phase of the moon in ASCII art.

[![asciicast](https://asciinema.org/a/760883.svg)](https://asciinema.org/a/760883)

This project is heavily inspired by the beautiful website [asciimoon.com](https://asciimoon.com/).

---

## Features

*   **Real-time Moon Phase:** Displays an ASCII art representation of the moon's phase for any given date.
*   **Interactive TUI:**
    *   Use the **Left** and **Right** arrow keys to travel through time day-by-day.
    *   Toggle labels for major lunar features with the **'l'** key.
    *   Cycle through multiple languages for labels (English, Chinese, French, Japanese, and Spanish) with the **'L'** key.
    *   Toggle the information panel with the **'i'** key.
*   **Cross-Platform:** Works on Linux, macOS, and Windows.
*   **Lightweight:** It's a single, small, native binary.

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

Run the application directly:

```sh
ascii_moon
```

To see the moon on a specific date:

```sh
ascii_moon --date YYYY-MM-DD
```

### Controls

*   **<Left Arrow>**: Go back one day.
*   **<Right Arrow>**: Go forward one day.
*   **l**: Toggle labels for lunar features.
*   **L**: Cycle through languages for the labels.
*   **i**: Toggle the information panel.
*   **q** or **<Esc>**: Quit the application.

## License

This project is licensed under the MIT License.
