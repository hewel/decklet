# Guest App and QuickJS/TSX Staging

Decklet's first runtime slice proves the native Host before it runs a live Guest App. This document records the intended Guest App boundary so future QuickJS and TSX work targets the existing Scene Snapshot model instead of inventing a browser-like surface.

## Plain-Object TSX Authoring

The planned TSX authoring model is a small JSX factory that returns plain JavaScript objects. A Guest App should describe native UI intent with component names such as `Screen`, `View`, `Text`, `Button`, `List`, and `Image`; it should not create DOM nodes or depend on browser layout.

At the boundary, TSX output is expected to be serializable into keyed Scene Snapshots:

- each node has a stable guest-supplied key;
- each node names one native component type;
- props carry native layout, visual, text, image, and focus intent;
- children are nested only where the native component model allows them.

The TSX layer is authoring sugar over the Scene Snapshot shape. It is not a React DOM target, a compatibility layer for browser components, or a general CSS runtime.

## Future QuickJS Boundary

QuickJS will host one foreground Guest App bundle. The Guest App will own its application state and produce a new Scene Snapshot after state changes. The Rust Host will own retained scene state, validation, layout, rendering, input normalization, and focus.

The intended loop is:

1. QuickJS evaluates the Guest App bundle.
2. The Guest App returns plain-object UI intent.
3. The runtime converts that object graph into a Scene Snapshot.
4. The Host ingests the Scene Snapshot and updates retained layout/focus state.
5. Host input produces intent-level events that are delivered back to the Guest App.

This staging keeps the Host hot path native and deterministic. QuickJS does not receive raw SDL events, layout internals, draw commands, or mutable Host state.

## Scene Snapshot Shape

Conceptually, a Scene Snapshot is a single rooted native UI tree. It is keyed, declarative, and describes the current UI state for one foreground Guest App.

A snapshot contains:

- a root screen node sized for the target viewport;
- container nodes for native stack layout;
- text, button, image, and list nodes with typed props;
- stable keys for identity, focus retention, and intent targets;
- typed visual values such as native colors and theme-token-derived values.

The Host treats a Scene Snapshot as input to ingestion. It lays out the tree, derives focus order from focusable keyed nodes, retains the current focused key when possible, and renders through the active backend.

## Host-to-Guest Intents

Input is normalized by the Host into gamepad-style actions, then emitted toward the Guest App as intent-level events. The initial vocabulary is deliberately small:

- focus changes when D-pad traversal moves to another keyed node;
- activate events for the focused node when A or Start is pressed;
- back events for B-style navigation;
- menu/debug events for Select-style actions.

Future Guest App handlers should react to these intents and return updated UI state. They should not depend on raw keyboard scancodes, SDL event structs, browser input events, or device-specific gamepad details.

## Explicitly Out of Scope

The staged Guest App contract does not introduce browser APIs, DOM, CSSOM, WebView behavior, React DOM compatibility, full CSS parsing, Tailwind compatibility, or emulator frontend compatibility.

QuickJS execution, TSX compilation, and live Guest App loading remain future slices. The visible Runtime Capability Demo remains Rust-built until those slices are explicitly implemented.
