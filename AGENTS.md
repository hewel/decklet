# Decklet Agent Guide

Decklet is a lightweight JSX-driven native UI runtime for small handheld Linux devices. It is a standalone project next to the dArkOS fork; do not place Decklet source directly inside the dArkOS repo.

## Project identity

Decklet is not a browser, React DOM, WebView, or emulator frontend. JS/TSX describes UI state and intent, while Rust owns the retained scene tree, layout, rendering, input dispatch, and runtime hot path.

## Architecture direction

- Rust native host and retained UI core.
- QuickJS guest runtime with a JSX/TSX app bundle.
- SDL2 backend for desktop development.
- SDL2 KMSDRM OpenGLES2 backend for RK3326/dArkOS deployment.
- Small native component model: Screen, View, Text, Button, List, Image.
- D-pad focus navigation with A/Start activate, B back, and Select debug/menu.
- No DOM, CSSOM, browser APIs, WebView, X11/Wayland device requirement, wgpu, or iced initially.

## Device and dArkOS boundaries

dArkOS integration comes later as a thin packaging/launch layer. The runtime should eventually install under `/opt/decklet`, with a systemd-compatible service disabled by default until manually verified.

Device service environment should use:

```sh
SDL_VIDEODRIVER=kmsdrm
SDL_RENDER_DRIVER=opengles2
SDL_VIDEO_EGL_DRIVER=libEGL.so
```

Hard rules:

- Do not modify U-Boot, boot partition files, `Image`, `uInitrd`, or DTB files.
- Do not install or upgrade kernel packages.
- Do not run `apt full-upgrade`.
- Do not write to block devices.
- Do not modify the stable Debian card or recovery image.
- Do not delete dArkOS generated images or large artifacts without asking.
- Do not enable any GUI systemd service by default until runtime behavior is verified.

## Initial runtime goal

Start with a maintainable 320x480 demo menu that opens on desktop SDL2, supports keyboard input, and routes input through a gamepad-style abstraction preparing for D-pad/A/B/Start/Select on device. Prefer small custom pieces over large dependencies until the minimal host works.

## Agent skills

### Issue tracker

Issues and PRDs are tracked in GitHub Issues; external PRs are not a triage request surface. See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default triage labels: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

This repo uses a single-context domain-doc layout. See `docs/agents/domain.md`.
