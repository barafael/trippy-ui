# trippy-ui

**Trace network paths like a normal person.**

`trippy-ui` is a windowed port of [trippy](https://github.com/fujiapple852/trippy),
the terminal traceroute and ping tool by [@fujiapple852](https://github.com/fujiapple852).
It keeps trippy's concept, its columns, its settings and its keyboard-driven
interface, but renders them with native [egui](https://github.com/emilk/egui)
widgets in a desktop window.

The data layer is trippy itself: `trippy-ui` drives `trippy-core`, `trippy-dns`
and `trippy-privilege`, so the tracing is exactly the tracing you get from `trip`.

## Features

| | |
|---|---|
| **Hops table** | All 28 trippy columns, with the terminal frontend's defaults and formatting. `Enter` toggles the selected hop's details. |
| **Chart** | Round-trip time of every hop over time, with zoom. |
| **History & histogram** | A sparkline of the selected hop's samples and its frequency distribution. |
| **Map** | The GeoIP locations of the hops on a world map (needs a MaxMind database). |
| **Flows** | The ECMP flows of a trace, for the Paris and Dublin multipath strategies. |
| **Settings** | The terminal frontend's tabs, made editable: view, trace, DNS, GeoIp, bindings, theme and columns. |
| **Themes** | Default, Gruvbox, Solarized and Catppuccin, light and dark, with per-colour overrides. |
| **Multiple targets** | One trace per target, switched with the tabs or the arrow keys. |

## Keys

| Key | Action |
|---|---|
| `↑` `↓` / `j` `k` | Move the selected hop |
| `←` `→` / `Tab` | Next / previous trace |
| `PgUp` `PgDn`, `Ctrl+u` `Ctrl+d` | Move by five |
| `Home` `End` | First / last hop |
| `c` `m` `f` | Chart / map / flows |
| `=` `-` | Zoom the chart |
| `d` / `Enter` | Hop details |
| `,` `.` | Cycle the addresses of the selected hop |
| `i` `n` `b` | Address mode: IP / hostname / both |
| `z` | Toggle autonomous system lookups |
| `p` `o` | Hide one more / fewer hop from the top |
| `[` `]` `{` `}` | Fewer / more / one / all addresses per hop |
| `/` | Filter the hops |
| `y` | Copy the selected hop |
| `s` | Settings (`1`–`7` jump to a tab) |
| `h` `?` | Help |
| `Ctrl+f` | Freeze the data |
| `Ctrl+r` `Ctrl+k` | Clear the trace / the DNS cache |
| `Esc` | Clear the selection, or close an overlay |
| `q` / `Ctrl+c` | Quit |

The status bar always shows the keys for the current view, and they are
clickable.

## Building

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

On Linux, trippy needs `CAP_NET_RAW` to open raw sockets. The simplest way to
grant it to the binary is:

```sh
sudo setcap cap_net_raw+p target/release/trippy-ui
```

On macOS and Windows, pass `-u` to use unprivileged mode.

## Command line

The command line follows trippy for everything that affects tracing and the
view. Run `trippy-ui --help` for the full list; the most useful options are:

```
trippy-ui [OPTIONS] <TARGET>...
  -p, --protocol <icmp|udp|tcp>
  -4, --ipv4 / -6, --ipv6 / -u, --unprivileged
  -R, --multipath-strategy <classic|paris|dublin>
  -a, --tui-address-mode <ip|host|both>
      --tui-custom-columns <COLUMNS>
      --tui-geoip-mode <off|short|long|location>
  -G, --geoip-mmdb-file <PATH>
```

Report modes (`--mode csv` and friends) are not ported; `trippy-ui` is the
interactive view.

## License

Apache-2.0, like trippy. The world map coastline is from the public-domain
dataset distributed with ratatui (MIT).
