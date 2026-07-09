use std::fmt;

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

    match value
        .get("schemaVersion")
        .and_then(serde_json::Value::as_u64)
    {
        Some(1) => {}
        Some(found) => {
            return Err(ContractError::UnsupportedSchemaVersion {
                found: found.to_string(),
            });
        }
        None => return Err(ContractError::MissingSchemaVersion),
    }

    let snapshot: WireSnapshot =
        serde_json::from_value(value).map_err(|error| ContractError::InvalidJson {
            message: error.to_string(),
        })?;
    Ok(SceneSnapshot::new(snapshot.root.into_scene_node()?))
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
    MissingRequiredProp {
        component_type: &'static str,
        prop: &'static str,
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
            Self::MissingRequiredProp {
                component_type,
                prop,
            } => write!(
                f,
                "{component_type} scene snapshot contract node requires props.{prop}"
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
    fn into_scene_node(self) -> Result<SceneNode, ContractError> {
        let component = self.component_type.into_component(self.props)?;
        let children = self
            .children
            .into_iter()
            .map(WireNode::into_scene_node)
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
