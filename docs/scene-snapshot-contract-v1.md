# Scene Snapshot Contract v1

The `Scene Snapshot Contract` is Decklet's strict serialized boundary for future Guest App output. It is parsed by `decklet_core::parse_scene_snapshot_contract`, converted into a native `SceneSnapshot`, and then passed to the existing `Host::ingest(snapshot, viewport)` API.

## Required Shape

Every v1 document must have a top-level `schemaVersion: 1` marker and a `root` node.

Each node uses this shape:

```json
{
  "key": "stable-node-key",
  "type": "screen",
  "props": {
    "layout": {
      "direction": "vertical",
      "padding": { "top": 0, "right": 0, "bottom": 0, "left": 0 },
      "spacing": 0,
      "size": { "width": 320, "height": 480 },
      "align": "stretch"
    },
    "visual": {
      "background": { "r": 26, "g": 31, "b": 40, "a": 255 },
      "foreground": { "r": 245, "g": 247, "b": 250, "a": 255 }
    }
  },
  "children": []
}
```

The supported `type` values are `screen`, `view`, `list`, `text`, `button`, and `image`. Only `screen`, `view`, and `list` are containers. `text`, `button`, and `image` must have empty `children` arrays.

All JSON fields are camelCase. Colors are explicit RGBA objects with integer channels from 0 through 255:

```json
{
  "r": 46,
  "g": 125,
  "b": 172,
  "a": 255
}
```

## Validation Guarantees

The parser rejects malformed or ambiguous Guest App output before Host ingestion. Validation rejects missing or unsupported schema versions, unknown fields, empty keys, duplicate keys anywhere in one snapshot, invalid component types, component props that do not belong to the node type, invalid color channels, and children on leaf components.

Keys are globally unique within a snapshot. This keeps retained focus, activation targets, and future Host-to-Guest intent events unambiguous.

The only defaults currently accepted by the parser are covered by contract tests: omitted container `focusable` defaults to `false`, omitted button `focusable` defaults to `true`, omitted `focusedBackground` and `border` default to `None`, and nullable or omitted `size.width` / `size.height` maps to native flexible size.

## Fixtures

`crates/decklet-core/fixtures/minimal.v1.json` is the minimal contract evidence fixture. It proves the required v1 marker, node shape, camelCase fields, RGBA colors, text props, button props, and Host ingestion seam.

`crates/decklet-core/fixtures/runtime-capability-demo.v1.json` is the full Runtime Capability Demo contract fixture. It mirrors the existing 320x480 demo at the contract level and proves the expected focusable menu row order through the public parser and Host ingestion.

The visible desktop demo still uses the Rust builder in `crates/decklet-demo/src/lib.rs` and `crates/decklet-demo/src/main.rs`. The runtime does not load these JSON fixtures in this slice.

## Out of Scope

QuickJS execution, TSX compilation, fixture-driven runtime loading, KMSDRM/device probing, dArkOS service packaging, and systemd service enablement remain out of scope for this contract slice.
