use decklet_core::{
    Align, ButtonProps, Color, Component, ContainerProps, EdgeInsets, LayoutProps, SceneNode,
    SceneSnapshot, SizeSpec, TextProps, VisualProps,
};

pub fn runtime_capability_demo_snapshot() -> SceneSnapshot {
    let theme = DemoTheme::default();
    SceneSnapshot::new(
        SceneNode::new(
            "demo-root",
            Component::Screen(ContainerProps::new(
                LayoutProps {
                    padding: EdgeInsets {
                        top: 20,
                        right: 16,
                        bottom: 16,
                        left: 16,
                    },
                    spacing: 12,
                    size: SizeSpec {
                        width: Some(320),
                        height: Some(480),
                    },
                    align: Align::Stretch,
                    ..LayoutProps::default()
                },
                VisualProps::new(theme.screen, theme.text),
            )),
        )
        .with_children(vec![
            title("Decklet Runtime"),
            subtitle("320x480 native Host demo"),
            menu_row("menu-play", "Open Library", theme),
            menu_row("menu-settings", "Settings", theme),
            menu_row("menu-debug", "Debug Panel", theme),
            menu_row("menu-power", "Power Options", theme),
        ]),
    )
}

fn title(text: &str) -> SceneNode {
    SceneNode::new(
        "demo-title",
        Component::Text(TextProps {
            text: text.to_owned(),
            layout: LayoutProps {
                size: SizeSpec {
                    width: None,
                    height: Some(26),
                },
                ..LayoutProps::default()
            },
            visual: VisualProps::new(Color::TRANSPARENT, Color::WHITE),
        }),
    )
}

fn subtitle(text: &str) -> SceneNode {
    SceneNode::new(
        "demo-subtitle",
        Component::Text(TextProps {
            text: text.to_owned(),
            layout: LayoutProps {
                size: SizeSpec {
                    width: None,
                    height: Some(22),
                },
                ..LayoutProps::default()
            },
            visual: VisualProps::new(Color::TRANSPARENT, Color::MUTED),
        }),
    )
}

fn menu_row(key: &'static str, label: &str, theme: DemoTheme) -> SceneNode {
    SceneNode::new(
        key,
        Component::Button(ButtonProps {
            label: label.to_owned(),
            layout: LayoutProps {
                size: SizeSpec {
                    width: None,
                    height: Some(44),
                },
                ..LayoutProps::default()
            },
            visual: theme.row_visual(),
            focusable: true,
        }),
    )
}

#[derive(Clone, Copy)]
struct DemoTheme {
    screen: Color,
    row: Color,
    focus: Color,
    text: Color,
}

impl DemoTheme {
    fn row_visual(self) -> VisualProps {
        VisualProps {
            background: self.row,
            foreground: self.text,
            focused_background: Some(self.focus),
            border: Some(Color::rgba(255, 255, 255, 38)),
        }
    }
}

impl Default for DemoTheme {
    fn default() -> Self {
        Self {
            screen: Color::INK,
            row: Color::PANEL,
            focus: Color::FOCUS,
            text: Color::WHITE,
        }
    }
}

#[cfg(test)]
mod tests {
    use decklet_core::{DEMO_VIEWPORT, Host, NodeKey};

    use super::*;

    #[test]
    fn demo_scene_has_stable_focusable_menu_rows() {
        let mut host = Host::new();
        let scene = host
            .ingest(runtime_capability_demo_snapshot(), DEMO_VIEWPORT)
            .expect("demo scene should ingest");

        assert_eq!(
            scene.focus_order,
            vec![
                NodeKey::from("menu-play"),
                NodeKey::from("menu-settings"),
                NodeKey::from("menu-debug"),
                NodeKey::from("menu-power"),
            ]
        );
    }
}
