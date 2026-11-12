# TopCalc

TopCalc is a small offline desktop launcher for Windows, macOS, and Linux. One process stays resident in the system tray. A global hotkey only shows a hidden window — it does not start a new process.

It is a quick search bar, a calculator, and an original 2D function grapher. It can also run a shell command or open that command in a terminal. Nothing in the app calls the network. Choosing **Search the web** opens your default browser.

The grapher is not GeoGebra and not Desmos. It does not reuse their source, assets, or names. GeoGebra is a large GPL Java desktop application, which is the opposite of a lightweight native launcher, so TopCalc ships its own small expression parser and plotter instead. The license is MIT.

## Install

Download the installer for your computer from the [TopCalc releases](https://github.com/nikolaK21/topcalc/releases) page and run it. Windows uses the file ending in `setup.exe`. After it is installed, TopCalc stays in the tray and **Alt+Space** opens it.

To build that installer yourself, open PowerShell in this folder and run:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\setup.ps1
```

On macOS or Linux:

```bash
bash scripts/setup.sh
```

The script installs Rust when it is missing, installs Node.js on Windows when that is missing, and builds an installer. The first build can take several minutes. The finished installer is under `src-tauri/target/release/bundle`.

## Hotkey

The default shortcut is **Alt+Space**. Cmd+Space fights Spotlight, and Ctrl+Space fights many input methods.

On Windows, Alt+Space is also the shortcut for a focused window's system menu. If the hotkey does not fire, change it.

Open TopCalc from the tray icon, then press **Ctrl+,** or the gear:

- type a shortcut such as `Alt+Space` or `Ctrl+Alt+Space`
- or press **Record** and hit the keys (recording suspends the current shortcut so it does not hide the window)

Settings are a JSON file in the app config directory:

- Windows: `%APPDATA%\com.topcalc.app\settings.json`
- macOS: `~/Library/Application Support/com.topcalc.app/settings.json`
- Linux: `~/.config/com.topcalc.app/settings.json`

```json
{
  "hotkey": "Ctrl+Alt+Space",
  "searchUrl": "https://duckduckgo.com/?q={query}",
  "theme": "dark",
  "anchor": "top"
}
```

`searchUrl` must be `http` or `https`. `{query}` or `%s` is replaced with the URL-encoded text.

**Color scheme** is Dark, Light, Slate, Forest, or Custom. Custom sets the background, text, muted text, accent, and surface colors.

**Position** is where the bar appears: Top, Center, Bottom, a corner, or Custom. Custom uses a horizontal and vertical percent, and the bar is centered on that point. Top is the middle of the screen, a little below the top edge. The bar moves while you change it. **Save** keeps the choice. **Esc** puts the last saved scheme and position back.

Launch at login is not implemented yet.

## Query syntax

The bar is empty until you type. One hint line lists the forms below. A single-file copy of this guide, for a phone or another computer, is [docs/TopCalc.html](docs/TopCalc.html) and [docs/TopCalc.pdf](docs/TopCalc.pdf).

| Type this | What happens |
| --- | --- |
| `2+2`, `sqrt(2)`, `sin(pi/2)`, or `=1+1` | Evaluated locally. Enter on the result copies it. |
| `sin(x)`, `y = x^2`, `x = 3`, `x^2 = 4` | Opens the graph. Equations also list real solutions, and the parsed input is shown as typeset math. Several expressions are separated with `;`. |
| `d/dx cos x`, `derivative of cos x`, `int sinx dx = y`, `int sinx dx from 0 to pi` | Derivative or integral, including `d^2/dx^2` and `y = int ...`. Indefinite integrals are graphed with `C = 0`. |
| Ctrl+G | Shows or hides the graph for the current math, including a constant plotted as a horizontal line. |
| `c:`, `d:`, `c:\Users`, `/`, `~` | File search under that root. `c:` and `d:` mean the drive root. |
| `chrome` (or any app name) | Installed applications, then files when the query is at least 3 characters and not an exact app name. |
| `> Get-Process` | Shell. On Windows `>` is PowerShell. |
| `$ ls` | On Windows, bash via Git Bash or WSL when one of those is installed. Otherwise the row says bash was not found. |
| `y = x^2`, `x = 3`, or `=sin(x)` | Opens a graph window. Enter in that window adds another line, and every line is drawn on the same graph. A leading `=` opens the window as soon as the math is valid. |
| `google.com`, `https://example.com/docs` | Enter opens that site. A leading space searches instead: ` google.com` searches for `google.com`. |
| `sleep` | Enter puts the computer to sleep. A leading space searches instead. |

On macOS and Linux, both `>` and `$` use your login shell (`$SHELL`, or `/bin/sh`).

### Calculator

Supported: `+ - * / ^ **`, parentheses, juxtaposition (`2x`, `sinx`, `sin2x`), and the functions on the ∑ keyboard: trig, inverse trig (`arcsin`), hyperbolic, `sqrt`, `ln`, `log`, `log10`, `log2`, `exp`, `abs`, `floor`, `ceil`, `round`, `sign`, and factorial (`5!`). `^` is right-associative, so `2^3^2` is 512. `-3^2` is `-9`. Trig is in radians. `log` and `ln` are natural log. `sin^2(x)` means `(sin(x))^2`.

Typing `sum`, `prod`, or `lim` followed by a space inserts that operator with the cursor in the body. `sum_{n=1}^{10} n` is 55. The ∑ button opens the same operators and the function keys, one group at a time: Trig, Inverse, Hyperbolic, Calculus, Functions, and Symbols.

Leading words such as `solve`, `plot`, `what is`, and `calculate` are ignored. Functions take parentheses, a space, or a following factor: `arcsin(1)`, `arcsin 1`, or `arcsinx`.

This is not a full computer-algebra system. There is no 3D and no implicit plots such as `x^2 + y^2 = 1`. It does solve real equations in `x`: `x = 3` is a vertical line, and `x^2 = 4` plots both sides and lists `x = -2` and `x = 2`. Exact results are used for lines and quadratics; other equations are solved numerically on [-20, 20]. `y = ...` and bare expressions in `x` are plotted as `y = f(x)`.

The graph is a shared Cartesian plane. Drag to pan, scroll to zoom, double-click or **Reset** to restore the view. Hover shows an approximate point. Discontinuities such as `1/x` and `tan(x)` break the line instead of drawing across the asymptote.

### Shell safety

Typing a command never runs it. The row shows the exact command. The first Enter on **Run in shell** only arms it (`Press Enter to run: ...`). A second Enter runs it. Arrowing onto that row and pressing Enter also runs it, because you highlighted it on purpose. **Open in terminal** does not need the extra confirm.

The inline runner captures stdout and stderr, waits at most 8 seconds, then kills the process and tells you to open a terminal. On Windows the inline shell is PowerShell with `-NoProfile`. Bash is used only when Git Bash or a WSL distro is actually present.

**Open in terminal** uses Windows Terminal (`wt`) when it is on `PATH`, otherwise PowerShell. On macOS it uses Terminal.app. On Linux it tries `x-terminal-emulator` and then common terminals. The command is handed to that terminal; TopCalc does not pretend bash exists when it does not.

### Files and apps

Applications come from the Start Menu (`.lnk`) on Windows, `/Applications`, `/System/Applications`, and `~/Applications` on macOS, and `.desktop` files in the usual XDG directories on Linux.

File search never walks a whole drive on each keystroke. Path queries are debounced by about 80 ms and capped (depth 3, a few thousand directory visits, 30 results). `c:` alone lists the top of that drive. These directory names are not entered unless you typed a path into them: `Windows`, `node_modules`, `.git`, plus `$Recycle.Bin`, `System Volume Information`, and on Unix `proc`, `sys`, and `dev`.

Without a path prefix, files are searched only after 3 characters, and only under Desktop, Documents, Downloads, and a few other home folders — not the whole disk. Ranking prefers exact and prefix matches. Applications outrank files with the same name.

Enter on an app launches it. Enter on a file or folder opens it with the default handler.

## Build and run

If Node.js and Rust are already installed, these do the same work as the setup script:

```bash
npm install
npm run tauri dev
```

`npm run tauri dev` builds the Rust host and serves the UI. The window stays hidden until **Alt+Space** or a left-click on the tray icon. `npm run tauri build` writes the installer.

WebView2 is already part of Windows 11. macOS uses the system WebKit and the Xcode Command Line Tools. Debian and Ubuntu need the packages installed by `scripts/setup.sh`.

Check the UI and the Rust core without launching the window:

```bash
npm run check
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```

## Project layout

- `src/` — Svelte UI (the bar, results, and graph canvas)
- `src-tauri/` — Tauri 2 / Rust: expression parser, file and app search, shell runner, window and hotkey
- `.github/workflows/ci.yml` — frontend typecheck and build, plus `cargo check` and `cargo test` on Ubuntu and Windows

## Limits

- One 2D explicit plotter. No CAS, no 3D, no GeoGebra file compatibility.
- File search is capped and skips the heavy directories listed above.
- Shell confirm and the 8 second inline timeout are intentional.
- The global shortcut depends on the OS allowing it. Alt+Space can lose to the Windows system menu; change it in settings if that happens.
- Launch at login is left for later.
