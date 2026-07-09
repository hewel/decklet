# Decklet

Decklet is a lightweight JSX-driven native UI runtime for small handheld Linux devices. This glossary names the runtime boundary and avoids browser, launcher, and emulator-frontend language for the core project.

## Language

**Decklet Runtime**:
The native runtime that hosts one foreground guest app bundle on a small-screen Linux handheld or desktop development target.
_Avoid_: Launcher, frontend, browser, WebView, emulator frontend

**Host**:
The Rust-owned side of Decklet that owns retained UI state, layout, rendering, input normalization, and focus.
_Avoid_: Native shell, browser engine, renderer process

**Guest App**:
A JS/TSX app bundle that describes UI state and intent for one foreground experience running inside the Decklet Runtime.
_Avoid_: Web app, page, launcher entry

**Scene Snapshot**:
A declarative tree of native UI intent sent from a Guest App to the Host after app state changes.
_Avoid_: DOM tree, draw command stream, virtual DOM

**Focus Model**:
The Host-owned D-pad navigation model over stable, guest-keyed focusable nodes.
_Avoid_: Tab order, browser focus, raw input handling

**Runtime Capability Demo**:
The first 320x480 Guest App used to validate rendering, layout, focus, and input behavior without defining Decklet as a product shell.
_Avoid_: Launcher mock, home screen, emulator menu
