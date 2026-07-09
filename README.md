# Decklet

Decklet is a lightweight native UI runtime for small handheld Linux devices. The first slice is a Rust-owned Host path: core scene state, constrained layout, focus, SDL2 Canvas rendering, and a 320x480 Runtime Capability Demo.

Decklet is not a browser, WebView, React DOM target, launcher, emulator frontend, or dArkOS image bring-up project. QuickJS and TSX are intentionally staged behind the native Host contract.

## Workspace

- `decklet-core`: keyed Scene Snapshots, retained Host state, constrained vertical stack layout, focus, and intent-level input behavior.
- `decklet-host-sdl`: SDL2 Canvas desktop/device host configuration, keyboard-to-gamepad input mapping, and SDL2_ttf text rendering.
- `decklet-demo`: the 320x480 Runtime Capability Demo scene and binary.

## Desktop Demo

The desktop demo requires the SDL2 runtime library and SDL2_ttf runtime library. On Arch-based systems the SDL2_ttf package is typically `sdl2_ttf`.

Run the desktop demo with:

```sh
cargo run -p decklet-demo
```

Desktop mode is the default. The demo opens a 320x480 SDL2 window, renders real text with SDL2_ttf, highlights the focused row, and logs intent-level events for activate, back, start, and debug/menu actions.

Keyboard mapping:

- Arrow keys: D-pad movement.
- Space or `Z`: A / activate.
- Backspace or `B`: B / back.
- Enter: Start.
- Tab or F1: Select / debug menu.

## Font Loading

`decklet-host-sdl` checks `DECKLET_FONT_PATH` first. If it is unset, it checks these fallback paths in order:

```text
/usr/share/fonts/TTF/DejaVuSans.ttf
/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf
/usr/share/fonts/liberation/LiberationSans-Regular.ttf
/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf
/opt/decklet/fonts/DejaVuSans.ttf
```

Example:

```sh
DECKLET_FONT_PATH=/usr/share/fonts/TTF/DejaVuSans.ttf cargo run -p decklet-demo
```

## Device Mode Configuration

Device mode is explicit configuration only in this slice. It does not install, enable, or write a systemd service.

Select device mode with either:

```sh
DECKLET_MODE=device cargo run -p decklet-demo
cargo run -p decklet-demo -- --device
```

Future RK3326/dArkOS launch should use:

```sh
SDL_VIDEODRIVER=kmsdrm
SDL_RENDER_DRIVER=opengles2
SDL_VIDEO_EGL_DRIVER=libEGL.so
```

## dArkOS Safety Boundaries

This repo must not:

- Modify U-Boot, boot partition files, `Image`, `uInitrd`, or DTB files.
- Install or upgrade kernel packages.
- Run `apt full-upgrade`.
- Write to block devices.
- Modify the stable Debian card or recovery image.
- Delete dArkOS generated images or large artifacts without asking.
- Enable any GUI systemd service by default before runtime behavior is manually verified.
