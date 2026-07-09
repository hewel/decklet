use std::fmt;

pub const DEMO_WIDTH: u32 = 320;
pub const DEMO_HEIGHT: u32 = 480;
pub const DEMO_VIEWPORT: Size = Size {
    width: DEMO_WIDTH,
    height: DEMO_HEIGHT,
};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeKey(String);

impl NodeKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for NodeKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for NodeKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for NodeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneSnapshot {
    pub root: SceneNode,
}

impl SceneSnapshot {
    pub fn new(root: SceneNode) -> Self {
        Self { root }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SceneNode {
    pub key: NodeKey,
    pub component: Component,
    pub children: Vec<SceneNode>,
}

impl SceneNode {
    pub fn new(key: impl Into<NodeKey>, component: Component) -> Self {
        Self {
            key: key.into(),
            component,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: impl Into<Vec<SceneNode>>) -> Self {
        self.children = children.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Component {
    Screen(ContainerProps),
    View(ContainerProps),
    List(ContainerProps),
    Text(TextProps),
    Button(ButtonProps),
    Image(ImageProps),
}

impl Component {
    fn kind(&self) -> ComponentKind {
        match self {
            Self::Screen(_) => ComponentKind::Screen,
            Self::View(_) => ComponentKind::View,
            Self::List(_) => ComponentKind::List,
            Self::Text(_) => ComponentKind::Text,
            Self::Button(_) => ComponentKind::Button,
            Self::Image(_) => ComponentKind::Image,
        }
    }

    fn layout(&self) -> LayoutProps {
        match self {
            Self::Screen(props) | Self::View(props) | Self::List(props) => props.layout,
            Self::Text(props) => props.layout,
            Self::Button(props) => props.layout,
            Self::Image(props) => props.layout,
        }
    }

    fn visual(&self) -> VisualProps {
        match self {
            Self::Screen(props) | Self::View(props) | Self::List(props) => props.visual,
            Self::Text(props) => props.visual,
            Self::Button(props) => props.visual,
            Self::Image(props) => props.visual,
        }
    }

    fn focusable(&self) -> bool {
        match self {
            Self::View(props) | Self::List(props) => props.focusable,
            Self::Button(props) => props.focusable,
            _ => false,
        }
    }

    fn text(&self) -> Option<&str> {
        match self {
            Self::Text(props) => Some(props.text.as_str()),
            Self::Button(props) => Some(props.label.as_str()),
            _ => None,
        }
    }

    fn default_height(&self) -> u32 {
        match self {
            Self::Screen(_) | Self::View(_) | Self::List(_) => 0,
            Self::Text(_) => 24,
            Self::Button(_) => 44,
            Self::Image(_) => 64,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ComponentKind {
    Screen,
    View,
    List,
    Text,
    Button,
    Image,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContainerProps {
    pub layout: LayoutProps,
    pub visual: VisualProps,
    pub focusable: bool,
}

impl ContainerProps {
    pub fn new(layout: LayoutProps, visual: VisualProps) -> Self {
        Self {
            layout,
            visual,
            focusable: false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextProps {
    pub text: String,
    pub layout: LayoutProps,
    pub visual: VisualProps,
}

impl TextProps {
    pub fn new(text: impl Into<String>, visual: VisualProps) -> Self {
        Self {
            text: text.into(),
            layout: LayoutProps::default(),
            visual,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ButtonProps {
    pub label: String,
    pub layout: LayoutProps,
    pub visual: VisualProps,
    pub focusable: bool,
}

impl ButtonProps {
    pub fn new(label: impl Into<String>, visual: VisualProps) -> Self {
        Self {
            label: label.into(),
            layout: LayoutProps {
                size: SizeSpec {
                    width: None,
                    height: Some(44),
                },
                ..LayoutProps::default()
            },
            visual,
            focusable: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImageProps {
    pub source: String,
    pub layout: LayoutProps,
    pub visual: VisualProps,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LayoutProps {
    pub direction: StackDirection,
    pub padding: EdgeInsets,
    pub spacing: u32,
    pub size: SizeSpec,
    pub align: Align,
}

impl Default for LayoutProps {
    fn default() -> Self {
        Self {
            direction: StackDirection::Vertical,
            padding: EdgeInsets::ZERO,
            spacing: 0,
            size: SizeSpec::default(),
            align: Align::Stretch,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StackDirection {
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Align {
    Start,
    Center,
    Stretch,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SizeSpec {
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EdgeInsets {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl EdgeInsets {
    pub const ZERO: Self = Self {
        top: 0,
        right: 0,
        bottom: 0,
        left: 0,
    };

    pub const fn all(value: u32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }

    pub const fn vertical_horizontal(vertical: u32, horizontal: u32) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VisualProps {
    pub background: Color,
    pub foreground: Color,
    pub focused_background: Option<Color>,
    pub border: Option<Color>,
}

impl VisualProps {
    pub const fn new(background: Color, foreground: Color) -> Self {
        Self {
            background,
            foreground,
            focused_background: None,
            border: None,
        }
    }
}

impl Default for VisualProps {
    fn default() -> Self {
        Self::new(Color::TRANSPARENT, Color::WHITE)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self::rgba(0, 0, 0, 0);
    pub const WHITE: Self = Self::rgb(245, 247, 250);
    pub const BLACK: Self = Self::rgb(10, 12, 16);
    pub const INK: Self = Self::rgb(26, 31, 40);
    pub const PANEL: Self = Self::rgb(37, 45, 57);
    pub const FOCUS: Self = Self::rgb(46, 125, 172);
    pub const MUTED: Self = Self::rgb(151, 163, 178);

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn inset(self, padding: EdgeInsets) -> Self {
        let horizontal = padding.left.saturating_add(padding.right);
        let vertical = padding.top.saturating_add(padding.bottom);
        Self {
            x: self.x + padding.left as i32,
            y: self.y + padding.top as i32,
            width: self.width.saturating_sub(horizontal),
            height: self.height.saturating_sub(vertical),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutTree {
    pub root: LayoutNode,
}

impl LayoutTree {
    pub fn find(&self, key: &NodeKey) -> Option<&LayoutNode> {
        self.root.find(key)
    }

    pub fn focus_order(&self) -> Vec<NodeKey> {
        let mut keys = Vec::new();
        self.root.collect_focusable(&mut keys);
        keys
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutNode {
    pub key: NodeKey,
    pub kind: ComponentKind,
    pub rect: Rect,
    pub focusable: bool,
    pub text: Option<String>,
    pub visual: VisualProps,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn find(&self, key: &NodeKey) -> Option<&LayoutNode> {
        if &self.key == key {
            return Some(self);
        }

        self.children.iter().find_map(|child| child.find(key))
    }

    pub fn visit(&self, visitor: &mut impl FnMut(&LayoutNode)) {
        visitor(self);
        for child in &self.children {
            child.visit(visitor);
        }
    }

    fn collect_focusable(&self, keys: &mut Vec<NodeKey>) {
        if self.focusable {
            keys.push(self.key.clone());
        }

        for child in &self.children {
            child.collect_focusable(keys);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RetainedScene {
    pub snapshot: SceneSnapshot,
    pub layout: LayoutTree,
    pub focus_order: Vec<NodeKey>,
    pub focused_key: Option<NodeKey>,
}

#[derive(Debug, Default)]
pub struct Host {
    scene: Option<RetainedScene>,
}

impl Host {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ingest(
        &mut self,
        snapshot: SceneSnapshot,
        viewport: Size,
    ) -> Result<&RetainedScene, HostError> {
        let layout = layout_snapshot(&snapshot, viewport)?;
        let focus_order = layout.focus_order();
        let previous_focus = self
            .scene
            .as_ref()
            .and_then(|scene| scene.focused_key.as_ref());
        let focused_key = previous_focus
            .filter(|key| focus_order.contains(key))
            .cloned()
            .or_else(|| focus_order.first().cloned());

        self.scene = Some(RetainedScene {
            snapshot,
            layout,
            focus_order,
            focused_key,
        });

        Ok(self.scene.as_ref().expect("scene was just ingested"))
    }

    pub fn scene(&self) -> Option<&RetainedScene> {
        self.scene.as_ref()
    }

    pub fn focused_key(&self) -> Option<&NodeKey> {
        self.scene
            .as_ref()
            .and_then(|scene| scene.focused_key.as_ref())
    }

    pub fn handle_action(&mut self, action: InputAction) -> Vec<HostIntent> {
        match action {
            InputAction::DpadUp | InputAction::DpadLeft => self.move_focus(-1),
            InputAction::DpadDown | InputAction::DpadRight => self.move_focus(1),
            InputAction::A | InputAction::Start => self
                .focused_key()
                .cloned()
                .map(|key| vec![HostIntent::Activate(key)])
                .unwrap_or_default(),
            InputAction::B => vec![HostIntent::Back],
            InputAction::Select => vec![HostIntent::MenuDebug],
        }
    }

    fn move_focus(&mut self, delta: isize) -> Vec<HostIntent> {
        let Some(scene) = self.scene.as_mut() else {
            return Vec::new();
        };
        if scene.focus_order.is_empty() {
            return Vec::new();
        }

        let current = scene
            .focused_key
            .as_ref()
            .and_then(|key| {
                scene
                    .focus_order
                    .iter()
                    .position(|candidate| candidate == key)
            })
            .unwrap_or(0);
        let max = scene.focus_order.len() as isize - 1;
        let next = (current as isize + delta).clamp(0, max) as usize;

        if next == current {
            return Vec::new();
        }

        let focused_key = scene.focus_order[next].clone();
        scene.focused_key = Some(focused_key.clone());
        vec![HostIntent::FocusChanged(focused_key)]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputAction {
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    A,
    B,
    Start,
    Select,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostIntent {
    FocusChanged(NodeKey),
    Activate(NodeKey),
    Back,
    MenuDebug,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostError {
    EmptyNodeKey,
}

impl fmt::Display for HostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyNodeKey => f.write_str("scene snapshots must use non-empty node keys"),
        }
    }
}

impl std::error::Error for HostError {}

pub fn layout_snapshot(snapshot: &SceneSnapshot, viewport: Size) -> Result<LayoutTree, HostError> {
    Ok(LayoutTree {
        root: layout_node(
            &snapshot.root,
            Rect::new(0, 0, viewport.width, viewport.height),
        )?,
    })
}

fn layout_node(node: &SceneNode, available: Rect) -> Result<LayoutNode, HostError> {
    if node.key.as_str().is_empty() {
        return Err(HostError::EmptyNodeKey);
    }

    let layout = node.component.layout();
    let rect = resolve_rect(&node.component, layout, available);
    let children = match node.component {
        Component::Screen(_) | Component::View(_) | Component::List(_) => {
            layout_vertical_children(&node.children, rect, layout)?
        }
        Component::Text(_) | Component::Button(_) | Component::Image(_) => Vec::new(),
    };

    Ok(LayoutNode {
        key: node.key.clone(),
        kind: node.component.kind(),
        rect,
        focusable: node.component.focusable(),
        text: node.component.text().map(str::to_owned),
        visual: node.component.visual(),
        children,
    })
}

fn layout_vertical_children(
    children: &[SceneNode],
    parent_rect: Rect,
    parent_layout: LayoutProps,
) -> Result<Vec<LayoutNode>, HostError> {
    let content = parent_rect.inset(parent_layout.padding);
    let mut y = content.y;
    let mut output = Vec::with_capacity(children.len());

    for child in children {
        let child_layout = child.component.layout();
        let child_height = child_layout
            .size
            .height
            .unwrap_or_else(|| child.component.default_height());
        let requested_width = child_layout.size.width.unwrap_or(content.width);
        let child_width = match parent_layout.align {
            Align::Stretch => content.width,
            Align::Start | Align::Center => requested_width.min(content.width),
        };
        let x = match parent_layout.align {
            Align::Start | Align::Stretch => content.x,
            Align::Center => content.x + ((content.width.saturating_sub(child_width)) / 2) as i32,
        };
        let available = Rect::new(x, y, child_width, child_height);
        output.push(layout_node(child, available)?);
        y += child_height.saturating_add(parent_layout.spacing) as i32;
    }

    Ok(output)
}

fn resolve_rect(component: &Component, layout: LayoutProps, available: Rect) -> Rect {
    Rect {
        x: available.x,
        y: available.y,
        width: layout.size.width.unwrap_or(available.width),
        height: layout
            .size
            .height
            .unwrap_or_else(|| available.height.max(component.default_height())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_ingests_keyed_scene_snapshot_without_sdl() {
        let mut host = Host::new();

        let scene = host
            .ingest(menu_scene(&["library", "settings"]), DEMO_VIEWPORT)
            .expect("snapshot should ingest");

        assert!(scene.layout.find(&NodeKey::from("root")).is_some());
        assert_eq!(
            scene.focus_order,
            vec![NodeKey::from("library"), NodeKey::from("settings")]
        );
        assert_eq!(scene.focused_key, Some(NodeKey::from("library")));
    }

    #[test]
    fn layout_for_320_by_480_menu_is_deterministic() {
        let mut host = Host::new();

        let scene = host
            .ingest(menu_scene(&["library", "settings", "power"]), DEMO_VIEWPORT)
            .expect("snapshot should lay out");

        assert_eq!(scene.layout.root.rect, Rect::new(0, 0, 320, 480));
        assert_eq!(
            scene
                .layout
                .find(&NodeKey::from("title"))
                .map(|node| node.rect),
            Some(Rect::new(16, 20, 288, 24))
        );
        assert_eq!(
            scene
                .layout
                .find(&NodeKey::from("library"))
                .map(|node| node.rect),
            Some(Rect::new(16, 56, 288, 44))
        );
        assert_eq!(
            scene
                .layout
                .find(&NodeKey::from("settings"))
                .map(|node| node.rect),
            Some(Rect::new(16, 112, 288, 44))
        );
    }

    #[test]
    fn focus_traversal_is_host_owned_and_key_stable() {
        let mut host = Host::new();
        host.ingest(menu_scene(&["library", "settings", "power"]), DEMO_VIEWPORT)
            .expect("snapshot should ingest");

        assert_eq!(
            host.handle_action(InputAction::DpadDown),
            vec![HostIntent::FocusChanged(NodeKey::from("settings"))]
        );

        host.ingest(menu_scene(&["library", "settings", "about"]), DEMO_VIEWPORT)
            .expect("snapshot should update");
        assert_eq!(host.focused_key(), Some(&NodeKey::from("settings")));

        assert_eq!(
            host.handle_action(InputAction::DpadUp),
            vec![HostIntent::FocusChanged(NodeKey::from("library"))]
        );
    }

    #[test]
    fn input_actions_emit_intent_level_events() {
        let mut host = Host::new();
        host.ingest(menu_scene(&["library"]), DEMO_VIEWPORT)
            .expect("snapshot should ingest");

        assert_eq!(
            host.handle_action(InputAction::A),
            vec![HostIntent::Activate(NodeKey::from("library"))]
        );
        assert_eq!(host.handle_action(InputAction::B), vec![HostIntent::Back]);
        assert_eq!(
            host.handle_action(InputAction::Select),
            vec![HostIntent::MenuDebug]
        );
    }

    fn menu_scene(keys: &[&str]) -> SceneSnapshot {
        let theme = ThemeTokens::default();
        let mut children = vec![SceneNode::new(
            "title",
            Component::Text(TextProps {
                text: "Decklet".to_owned(),
                layout: LayoutProps {
                    size: SizeSpec {
                        width: None,
                        height: Some(24),
                    },
                    ..LayoutProps::default()
                },
                visual: VisualProps::new(Color::TRANSPARENT, theme.text),
            }),
        )];

        children.extend(keys.iter().map(|key| {
            SceneNode::new(
                *key,
                Component::Button(ButtonProps {
                    label: key.to_string(),
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
        }));

        SceneSnapshot::new(
            SceneNode::new(
                "root",
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
            .with_children(children),
        )
    }

    #[derive(Clone, Copy)]
    struct ThemeTokens {
        screen: Color,
        row: Color,
        focus: Color,
        text: Color,
    }

    impl ThemeTokens {
        fn row_visual(self) -> VisualProps {
            VisualProps {
                background: self.row,
                foreground: self.text,
                focused_background: Some(self.focus),
                border: Some(Color::rgba(255, 255, 255, 32)),
            }
        }
    }

    impl Default for ThemeTokens {
        fn default() -> Self {
            Self {
                screen: Color::INK,
                row: Color::PANEL,
                focus: Color::FOCUS,
                text: Color::WHITE,
            }
        }
    }
}
