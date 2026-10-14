# trippy-ui

**Trace network paths like a normal person.**

A windowed port of [trippy](https://github.com/fujiapple852/trippy)'s terminal
frontend, built with [egui](https://github.com/emilk/egui). It reuses trippy's
data layer (`trippy-core`, `trippy-dns`, `trippy-privilege`), so the tracing is
exactly the tracing you get from `trip`, and keeps its columns, settings and
keyboard-driven navigation.

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

On macOS and Windows, pass `-u` to use unprivileged mode.

## Usage

The command line follows trippy for everything that affects tracing and the
view; run `trippy-ui --help` for the full list. The status bar always shows the
keys for the current view, and they are clickable.

## License

Apache-2.0, like trippy.
