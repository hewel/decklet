use std::{collections::HashSet, fmt};

use serde::Deserialize;

use crate::{
    Align, ButtonProps, Color, Component, ContainerProps, EdgeInsets, ImageProps, LayoutProps,
    SceneNode, SceneSnapshot, SizeSpec, StackDirection, TextProps, VisualProps,
};

pub fn parse_scene_snapshot_contract(json: &str) -> Result<SceneSnapshot, ContractError> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|error| ContractError::InvalidJson {
            message: error.to_string(),
        })?;

    match value.get("schemaVersion") {
        Some(version) if version.as_u64() == Some(1) => {}
        Some(found) => {
            return Err(ContractError::UnsupportedSchemaVersion {
                found: found.to_string(),
            });
        }
        None => return Err(ContractError::MissingSchemaVersion),
    }

    let snapshot: WireSnapshot =
        serde_json::from_value(value).map_err(|error| ContractError::InvalidContract {
            message: error.to_string(),
        })?;
    let mut keys = HashSet::new();
    Ok(SceneSnapshot::new(
        snapshot.root.into_scene_node(&mut keys)?,
    ))
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    InvalidJson {
        message: String,
    },
    MissingSchemaVersion,
    UnsupportedSchemaVersion {
        found: String,
    },
    InvalidContract {
        message: String,
    },
    EmptyNodeKey,
    DuplicateNodeKey {
        key: String,
    },
    MissingRequiredProp {
        component_type: &'static str,
        prop: &'static str,
    },
    UnexpectedProp {
        component_type: &'static str,
        prop: &'static str,
    },
    LeafNodeChildren {
        component_type: &'static str,
        key: String,
    },
}

impl fmt::Display for ContractError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson { message } => {
                write!(f, "invalid scene snapshot contract: {message}")
            }
            Self::MissingSchemaVersion => {
                f.write_str("scene snapshot contract requires schemaVersion: 1")
            }
            Self::UnsupportedSchemaVersion { found } => write!(
                f,
                "unsupported scene snapshot contract schemaVersion: {found}"
            ),
            Self::InvalidContract { message } => {
                write!(f, "invalid scene snapshot contract shape: {message}")
            }
            Self::EmptyNodeKey => {
                f.write_str("scene snapshot contract nodes require non-empty keys")
            }
            Self::DuplicateNodeKey { key } => write!(
                f,
                "scene snapshot contract node key appears more than once: {key}"
            ),
            Self::MissingRequiredProp {
                component_type,
                prop,
            } => write!(
                f,
                "{component_type} scene snapshot contract node requires props.{prop}"
            ),
            Self::UnexpectedProp {
                component_type,
                prop,
            } => write!(
                f,
                "{component_type} scene snapshot contract node does not accept props.{prop}"
            ),
            Self::LeafNodeChildren {
                component_type,
                key,
            } => write!(
                f,
                "{component_type} scene snapshot contract node cannot have children: {key}"
            ),
        }
    }
}

impl std::error::Error for ContractError {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireSnapshot {
    #[serde(rename = "schemaVersion")]
    _schema_version: u32,
    root: WireNode,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireNode {
    key: String,
    #[serde(rename = "type")]
    component_type: WireComponentType,
    props: WireProps,
    children: Vec<WireNode>,
}

impl WireNode {
    fn into_scene_node(self, keys: &mut HashSet<String>) -> Result<SceneNode, ContractError> {
        if self.key.is_empty() {
            return Err(ContractError::EmptyNodeKey);
        }
        if !keys.insert(self.key.clone()) {
            return Err(ContractError::DuplicateNodeKey { key: self.key });
        }
        if !self.component_type.accepts_children() && !self.children.is_empty() {
            return Err(ContractError::LeafNodeChildren {
                component_type: self.component_type.as_str(),
                key: self.key,
            });
        }

        let component = self.component_type.into_component(self.props)?;
        let children = self
            .children
            .into_iter()
            .map(|node| node.into_scene_node(keys))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(SceneNode::new(self.key, component).with_children(children))
    }
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WireComponentType {
    Screen,
    View,
    Text,
    Button,
    List,
    Image,
}

impl WireComponentType {
    fn accepts_children(self) -> bool {
        matches!(self, Self::Screen | Self::View | Self::List)
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Screen => "screen",
            Self::View => "view",
            Self::Text => "text",
            Self::Button => "button",
            Self::List => "list",
            Self::Image => "image",
        }
    }

    fn into_component(self, props: WireProps) -> Result<Component, ContractError> {
        props.validate_for(self)?;
        let WireProps {
            layout,
            visual,
            focusable,
            text,
            label,
            source,
        } = props;
        let layout = layout.into();
        let visual = visual.into();
        match self {
            Self::Screen => Ok(Component::Screen(ContainerProps {
                layout,
                visual,
                focusable: focusable.unwrap_or(false),
            })),
            Self::View => Ok(Component::View(ContainerProps {
                layout,
                visual,
                focusable: focusable.unwrap_or(false),
            })),
            Self::List => Ok(Component::List(ContainerProps {
                layout,
                visual,
                focusable: focusable.unwrap_or(false),
            })),
            Self::Text => Ok(Component::Text(TextProps {
                text: required_text(self, "text", text)?,
                layout,
                visual,
            })),
            Self::Button => Ok(Component::Button(ButtonProps {
                label: required_text(self, "label", label)?,
                layout,
                visual,
                focusable: focusable.unwrap_or(true),
            })),
            Self::Image => Ok(Component::Image(ImageProps {
                source: required_text(self, "source", source)?,
                layout,
                visual,
            })),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireProps {
    layout: WireLayoutProps,
    visual: WireVisualProps,
    focusable: Option<bool>,
    text: Option<String>,
    label: Option<String>,
    source: Option<String>,
}

impl WireProps {
    fn validate_for(&self, component_type: WireComponentType) -> Result<(), ContractError> {
        match component_type {
            WireComponentType::Screen | WireComponentType::View | WireComponentType::List => {
                self.reject_present(component_type, "text", self.text.as_ref())?;
                self.reject_present(component_type, "label", self.label.as_ref())?;
                self.reject_present(component_type, "source", self.source.as_ref())?;
            }
            WireComponentType::Text => {
                self.reject_present(component_type, "focusable", self.focusable.as_ref())?;
                self.reject_present(component_type, "label", self.label.as_ref())?;
                self.reject_present(component_type, "source", self.source.as_ref())?;
            }
            WireComponentType::Button => {
                self.reject_present(component_type, "text", self.text.as_ref())?;
                self.reject_present(component_type, "source", self.source.as_ref())?;
            }
            WireComponentType::Image => {
                self.reject_present(component_type, "focusable", self.focusable.as_ref())?;
                self.reject_present(component_type, "text", self.text.as_ref())?;
                self.reject_present(component_type, "label", self.label.as_ref())?;
            }
        }

        Ok(())
    }

    fn reject_present<T>(
        &self,
        component_type: WireComponentType,
        prop: &'static str,
        value: Option<&T>,
    ) -> Result<(), ContractError> {
        if value.is_some() {
            return Err(ContractError::UnexpectedProp {
                component_type: component_type.as_str(),
                prop,
            });
        }

        Ok(())
    }
}

fn required_text(
    component_type: WireComponentType,
    prop: &'static str,
    value: Option<String>,
) -> Result<String, ContractError> {
    value.ok_or(ContractError::MissingRequiredProp {
        component_type: component_type.as_str(),
        prop,
    })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireLayoutProps {
    direction: WireStackDirection,
    padding: WireEdgeInsets,
    spacing: u32,
    size: WireSizeSpec,
    align: WireAlign,
}

impl From<WireLayoutProps> for LayoutProps {
    fn from(value: WireLayoutProps) -> Self {
        Self {
            direction: value.direction.into(),
            padding: value.padding.into(),
            spacing: value.spacing,
            size: value.size.into(),
            align: value.align.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WireStackDirection {
    Vertical,
}

impl From<WireStackDirection> for StackDirection {
    fn from(value: WireStackDirection) -> Self {
        match value {
            WireStackDirection::Vertical => Self::Vertical,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WireAlign {
    Start,
    Center,
    Stretch,
}

impl From<WireAlign> for Align {
    fn from(value: WireAlign) -> Self {
        match value {
            WireAlign::Start => Self::Start,
            WireAlign::Center => Self::Center,
            WireAlign::Stretch => Self::Stretch,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireEdgeInsets {
    top: u32,
    right: u32,
    bottom: u32,
    left: u32,
}

impl From<WireEdgeInsets> for EdgeInsets {
    fn from(value: WireEdgeInsets) -> Self {
        Self {
            top: value.top,
            right: value.right,
            bottom: value.bottom,
            left: value.left,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireSizeSpec {
    width: Option<u32>,
    height: Option<u32>,
}

impl From<WireSizeSpec> for SizeSpec {
    fn from(value: WireSizeSpec) -> Self {
        Self {
            width: value.width,
            height: value.height,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireVisualProps {
    background: WireColor,
    foreground: WireColor,
    focused_background: Option<WireColor>,
    border: Option<WireColor>,
}

impl From<WireVisualProps> for VisualProps {
    fn from(value: WireVisualProps) -> Self {
        Self {
            background: value.background.into(),
            foreground: value.foreground.into(),
            focused_background: value.focused_background.map(Into::into),
            border: value.border.map(Into::into),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireColor {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl From<WireColor> for Color {
    fn from(value: WireColor) -> Self {
        Self::rgba(value.r, value.g, value.b, value.a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DEMO_VIEWPORT, Host, NodeKey, Rect};

    #[test]
    fn parses_minimal_v1_contract_snapshot_into_host() {
        let snapshot = parse_scene_snapshot_contract(MINIMAL_CONTRACT).expect("valid contract");
        let mut host = Host::new();
        let scene = host
            .ingest(snapshot, DEMO_VIEWPORT)
            .expect("contract snapshot should ingest");

        assert_eq!(scene.layout.root.rect, Rect::new(0, 0, 320, 480));
        assert_eq!(scene.focus_order, vec![NodeKey::from("primary-action")]);
        assert_eq!(scene.focused_key, Some(NodeKey::from("primary-action")));

        let button = scene
            .layout
            .find(&NodeKey::from("primary-action"))
            .expect("button layout exists");
        assert_eq!(button.text.as_deref(), Some("Open Library"));
        assert_eq!(
            button.visual.focused_background,
            Some(Color::rgba(46, 125, 172, 255))
        );
    }

    #[test]
    fn rejects_contract_without_schema_version_one() {
        let error = parse_scene_snapshot_contract(
            r#"{
              "schemaVersion": 2,
              "root": {
                "key": "root",
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
                    "background": { "r": 0, "g": 0, "b": 0, "a": 255 },
                    "foreground": { "r": 255, "g": 255, "b": 255, "a": 255 },
                    "focusedBackground": null,
                    "border": null
                  },
                  "focusable": false,
                  "text": null,
                  "label": null,
                  "source": null
                },
                "children": []
              }
            }"#,
        )
        .expect_err("unsupported schema version should fail");

        assert_eq!(
            error,
            ContractError::UnsupportedSchemaVersion {
                found: "2".to_owned()
            }
        );
    }

    #[test]
    fn rejects_missing_schema_version_with_typed_error() {
        let mut value = minimal_contract_value();
        value.as_object_mut().unwrap().remove("schemaVersion");

        assert_eq!(
            parse_value(value).expect_err("missing schema version should fail"),
            ContractError::MissingSchemaVersion
        );
    }

    #[test]
    fn rejects_non_numeric_schema_version_with_typed_error() {
        let mut value = minimal_contract_value();
        value["schemaVersion"] = serde_json::json!("1");

        assert_eq!(
            parse_value(value).expect_err("unsupported schema version should fail"),
            ContractError::UnsupportedSchemaVersion {
                found: "\"1\"".to_owned()
            }
        );
    }

    #[test]
    fn rejects_empty_keys_with_typed_error() {
        let mut value = minimal_contract_value();
        value["root"]["children"][0]["key"] = serde_json::json!("");

        assert_eq!(
            parse_value(value).expect_err("empty key should fail"),
            ContractError::EmptyNodeKey
        );
    }

    #[test]
    fn rejects_duplicate_keys_globally() {
        let mut value = minimal_contract_value();
        value["root"]["children"][1]["key"] = serde_json::json!("title");

        assert_eq!(
            parse_value(value).expect_err("duplicate key should fail"),
            ContractError::DuplicateNodeKey {
                key: "title".to_owned()
            }
        );
    }

    #[test]
    fn rejects_unknown_fields() {
        let mut value = minimal_contract_value();
        value["root"]["unexpectedField"] = serde_json::json!(true);

        let error = parse_value(value).expect_err("unknown field should fail");
        assert!(matches!(error, ContractError::InvalidContract { .. }));
        assert!(error.to_string().contains("unknown field"));
    }

    #[test]
    fn rejects_invalid_component_types() {
        let mut value = minimal_contract_value();
        value["root"]["children"][0]["type"] = serde_json::json!("slider");

        let error = parse_value(value).expect_err("invalid component type should fail");
        assert!(matches!(error, ContractError::InvalidContract { .. }));
        assert!(error.to_string().contains("unknown variant"));
    }

    #[test]
    fn rejects_malformed_props_for_component_type() {
        let mut value = minimal_contract_value();
        value["root"]["props"]["label"] = serde_json::json!("not allowed");

        assert_eq!(
            parse_value(value).expect_err("screen label prop should fail"),
            ContractError::UnexpectedProp {
                component_type: "screen",
                prop: "label"
            }
        );
    }

    #[test]
    fn rejects_invalid_color_channel_values() {
        let mut value = minimal_contract_value();
        value["root"]["props"]["visual"]["background"]["r"] = serde_json::json!(300);

        let error = parse_value(value).expect_err("invalid color should fail");
        assert!(matches!(error, ContractError::InvalidContract { .. }));
        assert!(error.to_string().contains("invalid value"));
    }

    #[test]
    fn rejects_children_on_leaf_components() {
        let mut value = minimal_contract_value();
        value["root"]["children"][0]["children"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!({
                "key": "illegal-child",
                "type": "text",
                "props": {
                    "layout": {
                        "direction": "vertical",
                        "padding": { "top": 0, "right": 0, "bottom": 0, "left": 0 },
                        "spacing": 0,
                        "size": { "width": null, "height": 20 },
                        "align": "stretch"
                    },
                    "visual": {
                        "background": { "r": 0, "g": 0, "b": 0, "a": 0 },
                        "foreground": { "r": 245, "g": 247, "b": 250, "a": 255 },
                        "focusedBackground": null,
                        "border": null
                    },
                    "text": "Child"
                },
                "children": []
            }));

        assert_eq!(
            parse_value(value).expect_err("leaf children should fail"),
            ContractError::LeafNodeChildren {
                component_type: "text",
                key: "title".to_owned()
            }
        );
    }

    #[test]
    fn accepts_only_tested_contract_defaults() {
        let mut value = minimal_contract_value();
        value["root"]["props"]
            .as_object_mut()
            .unwrap()
            .remove("focusable");
        value["root"]["props"]["visual"]
            .as_object_mut()
            .unwrap()
            .remove("focusedBackground");
        value["root"]["props"]["visual"]
            .as_object_mut()
            .unwrap()
            .remove("border");
        value["root"]["children"][1]["props"]
            .as_object_mut()
            .unwrap()
            .remove("focusable");

        let snapshot = parse_value(value).expect("tested defaults should parse");
        let mut host = Host::new();
        let scene = host
            .ingest(snapshot, DEMO_VIEWPORT)
            .expect("defaulted snapshot should ingest");

        assert!(!scene.layout.root.focusable);
        assert_eq!(scene.layout.root.visual.focused_background, None);
        assert_eq!(scene.layout.root.visual.border, None);
        assert_eq!(scene.focus_order, vec![NodeKey::from("primary-action")]);
    }

    fn minimal_contract_value() -> serde_json::Value {
        serde_json::from_str(MINIMAL_CONTRACT).expect("minimal contract fixture is valid JSON")
    }

    fn parse_value(value: serde_json::Value) -> Result<SceneSnapshot, ContractError> {
        parse_scene_snapshot_contract(&value.to_string())
    }

    const MINIMAL_CONTRACT: &str = r#"{
      "schemaVersion": 1,
      "root": {
        "key": "root",
        "type": "screen",
        "props": {
          "layout": {
            "direction": "vertical",
            "padding": { "top": 20, "right": 16, "bottom": 16, "left": 16 },
            "spacing": 12,
            "size": { "width": 320, "height": 480 },
            "align": "stretch"
          },
          "visual": {
            "background": { "r": 26, "g": 31, "b": 40, "a": 255 },
            "foreground": { "r": 245, "g": 247, "b": 250, "a": 255 },
            "focusedBackground": null,
            "border": null
          },
          "focusable": false,
          "text": null,
          "label": null,
          "source": null
        },
        "children": [
          {
            "key": "title",
            "type": "text",
            "props": {
              "layout": {
                "direction": "vertical",
                "padding": { "top": 0, "right": 0, "bottom": 0, "left": 0 },
                "spacing": 0,
                "size": { "width": null, "height": 24 },
                "align": "stretch"
              },
              "visual": {
                "background": { "r": 0, "g": 0, "b": 0, "a": 0 },
                "foreground": { "r": 245, "g": 247, "b": 250, "a": 255 },
                "focusedBackground": null,
                "border": null
              },
              "focusable": null,
              "text": "Decklet",
              "label": null,
              "source": null
            },
            "children": []
          },
          {
            "key": "primary-action",
            "type": "button",
            "props": {
              "layout": {
                "direction": "vertical",
                "padding": { "top": 0, "right": 0, "bottom": 0, "left": 0 },
                "spacing": 0,
                "size": { "width": null, "height": 44 },
                "align": "stretch"
              },
              "visual": {
                "background": { "r": 37, "g": 45, "b": 57, "a": 255 },
                "foreground": { "r": 245, "g": 247, "b": 250, "a": 255 },
                "focusedBackground": { "r": 46, "g": 125, "b": 172, "a": 255 },
                "border": { "r": 255, "g": 255, "b": 255, "a": 38 }
              },
              "focusable": true,
              "text": null,
              "label": "Open Library",
              "source": null
            },
            "children": []
          }
        ]
      }
    }"#;
}
