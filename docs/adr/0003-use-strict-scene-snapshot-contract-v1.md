# Use a Strict Scene Snapshot Contract v1

Decklet will define a strict, versioned `Scene Snapshot Contract` v1 as the serialized boundary between a future Guest App and the Rust Host. The contract is a separate wire DTO that validates required `schemaVersion: 1`, native component node shape, globally unique snapshot keys, typed props, and supported defaults before conversion into the core `Scene Snapshot`.

The Host's retained scene state, layout tree, rendering backend, input normalization, and focus model remain native Rust internals. The contract describes Guest App output, not the full Host behavior surface.

**Considered Options**

- Strict v1 wire DTO: accepted because it gives QuickJS/TSX a stable serialized target while letting the Host reject malformed or ambiguous Guest App output before ingestion.
- Reusing the core `Scene Snapshot` structs as the serialized format: rejected because serde concerns, versioning, defaults, and validation errors would leak into the retained Host model.
- Accepting loosely validated JSON: rejected because duplicate keys, unknown fields, invalid component types, malformed props, and invalid colors would make focus retention and Host errors ambiguous.
- Per-node local key uniqueness: rejected because focus targets and intent events need one globally unambiguous key namespace per snapshot.
- Adding QuickJS execution in this slice: deferred until the native contract parser and validation behavior are proven.
- Loading fixture JSON in the desktop demo runtime: deferred because fixtures are contract evidence for tests, while the visible Runtime Capability Demo remains Rust-built in this slice.
