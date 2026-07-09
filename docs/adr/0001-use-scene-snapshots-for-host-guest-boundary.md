# Use Scene Snapshots for the Host-Guest Boundary

Decklet will have JS/TSX Guest Apps produce keyed declarative Scene Snapshots after state changes, while the Rust Host owns retained scene state, layout, rendering, input dispatch, and focus. This keeps the boundary explicit and deterministic, rejects per-frame imperative draw calls from JS, and avoids leaking raw input plumbing into the guest side before the native runtime model is stable.

**Considered Options**

- Scene Snapshots: accepted because they keep JS focused on UI intent while Rust owns the hot path.
- Batched native operations: deferred because they expose host mutation details too early.
- Imperative draw calls: rejected because they contradict the retained-runtime goal and put frame hot-path work in JS.
