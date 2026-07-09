# Stage the Native SDL Host before QuickJS

Decklet will first prove the native Host with a Rust workspace split into `decklet-core`, `decklet-host-sdl`, and `decklet-demo`, using a native Rust scene before adding live QuickJS execution. The first backend should use SDL2 Canvas, constrained stack layout, typed props and theme tokens, host-owned focus, guest-supplied node keys, and intent-level events so the runtime semantics are clear before introducing the guest scripting runtime.

**Considered Options**

- Native SDL host first: accepted because it validates the runtime hot path before QuickJS adds boundary and dependency churn.
- QuickJS proof of concept immediately: deferred until scene, layout, input, and focus semantics are stable.
- SDL window only with no scene model: rejected because it would not prove the retained UI contract Decklet needs.
