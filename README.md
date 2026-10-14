# trippy-ui

**Trace network paths like a normal person.**

A windowed port of [trippy](https://github.com/fujiapple852/trippy)'s terminal
frontend, built with [egui](https://github.com/emilk/egui). It reuses trippy's
data layer (`trippy-core`, `trippy-dns`, `trippy-privilege`), so the tracing is
exactly the tracing you get from `trip`, and keeps its columns, settings,
config file and keyboard-driven navigation.

## Build

`trippy-ui` builds against a checkout of the trippy workspace, expected next to
this repository:

```sh
git clone https://github.com/fujiapple852/trippy ../trippy
cargo run --release -- example.com
```

On Linux the window needs the usual X11/Wayland development headers
(`libxkbcommon-dev libwayland-dev libxcb-render0-dev libxcb-shape0-dev
libxcb-xfixes0-dev`).

## Privileges

On Linux, trippy needs `CAP_NET_RAW` to open raw sockets:

```sh
sudo setcap cap_net_raw+p target/release/trippy-ui
```

On macOS and Windows, pass `-u` to use unprivileged mode. Like `trip`, the
window refuses to start without the privileges it needs.

## Configuration

The command line follows `trip` for everything that affects tracing and the
view; run `trippy-ui --help` for the full list. Every option can also be set
through the same `TRIP_*` environment variable `trip` reads (for example
`TRIP_TUI_ADDRESS_MODE=ip`), and through trippy's config file.

A `trippy.toml` (or `.trippy.toml`) is read from the first of these places that
has one: the current directory, your home directory, your config directory and
`<config directory>/trippy`; `-c` names a file explicitly. The command line wins
over the file, which wins over trippy's defaults, and the values are validated
with trippy's rules. The sections `[trippy]` (except `unprivileged`) and
`[report]` belong to `trip` and are accepted and ignored.

Key bindings use trippy's syntax, on the command line
(`--tui-key-bindings toggle-freeze=ctrl+g,toggle-chart=x`) or in the
`[bindings]` section. `--print-tui-binding-commands` lists the commands. The
window adds a few keys of its own on top: `Tab`/`Shift+Tab`, `j`/`k`/`l`,
`PgUp`/`PgDn`, `Home`/`End`, `Enter`, `y` to copy, `/` to filter and `+` to
zoom.

Theme colours use trippy's items and colour names, on the command line
(`--tui-theme-colors samples-chart-color=red,bg-color=#102030`) or in the
`[theme-colors]` section; `--print-tui-theme-items` lists the items. They are
applied on top of the palette chosen in the Theme tab (default, Gruvbox,
Solarized or Catppuccin, each light and dark). Items the window has no
counterpart for are accepted and ignored.

Not ported: the report modes (`--mode csv` and friends), the translated
interface (`--tui-locale` only affects GeoIP names) and `--tui-preserve-screen`.

## Usage

The status bar always shows the keys for the current view, and they are
clickable. `h` opens the help overlay with every key, `s` the settings.

## License

Apache-2.0, like trippy.
