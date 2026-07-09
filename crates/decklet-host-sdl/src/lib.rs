use std::env;
use std::error::Error;
use std::ffi::CString;
use std::fmt;
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::time::Duration;

use decklet_core::{
    Color, DEMO_VIEWPORT, Host, HostIntent, InputAction, LayoutNode, NodeKey, SceneSnapshot, Size,
};
use libloading::Library;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color as SdlColor;
use sdl2::rect::Rect as SdlRect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

pub const DEVICE_SDL_ENVIRONMENT: [(&str, &str); 3] = [
    ("SDL_VIDEODRIVER", "kmsdrm"),
    ("SDL_RENDER_DRIVER", "opengles2"),
    ("SDL_VIDEO_EGL_DRIVER", "libEGL.so"),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchMode {
    Desktop,
    Device,
}

impl LaunchMode {
    pub fn from_env_and_args() -> Self {
        if env::args().any(|arg| arg == "--device") {
            return Self::Device;
        }

        match env::var("DECKLET_MODE") {
            Ok(value) if value.eq_ignore_ascii_case("device") => Self::Device,
            _ => Self::Desktop,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SdlHostConfig {
    pub mode: LaunchMode,
    pub window_size: Size,
    pub title: String,
    pub font: FontPathPolicy,
}

impl SdlHostConfig {
    pub fn desktop() -> Self {
        Self {
            mode: LaunchMode::Desktop,
            window_size: DEMO_VIEWPORT,
            title: "Decklet Runtime Capability Demo".to_owned(),
            font: FontPathPolicy::default(),
        }
    }

    pub fn from_env_and_args() -> Self {
        Self {
            mode: LaunchMode::from_env_and_args(),
            ..Self::desktop()
        }
    }

    pub fn device_environment(&self) -> Option<&'static [(&'static str, &'static str); 3]> {
        (self.mode == LaunchMode::Device).then_some(&DEVICE_SDL_ENVIRONMENT)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FontPathPolicy {
    pub env_var: &'static str,
    pub fallback_paths: Vec<PathBuf>,
}

impl FontPathPolicy {
    pub fn resolve(&self) -> Result<PathBuf, SdlHostError> {
        if let Ok(value) = env::var(self.env_var) {
            let path = PathBuf::from(value);
            if path.exists() {
                return Ok(path);
            }

            return Err(SdlHostError::message(format!(
                "{} points to a font that does not exist: {}",
                self.env_var,
                path.display()
            )));
        }

        self.fallback_paths
            .iter()
            .find(|path| path.exists())
            .cloned()
            .ok_or_else(|| {
                SdlHostError::message(format!(
                    "no font found; set {} or install one of: {}",
                    self.env_var,
                    self.fallback_paths
                        .iter()
                        .map(|path| path.display().to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            })
    }
}

impl Default for FontPathPolicy {
    fn default() -> Self {
        Self {
            env_var: "DECKLET_FONT_PATH",
            fallback_paths: vec![
                PathBuf::from("/usr/share/fonts/TTF/DejaVuSans.ttf"),
                PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"),
                PathBuf::from("/usr/share/fonts/liberation/LiberationSans-Regular.ttf"),
                PathBuf::from("/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"),
                PathBuf::from("/opt/decklet/fonts/DejaVuSans.ttf"),
            ],
        }
    }
}

#[derive(Debug)]
pub struct SdlHostError {
    message: String,
}

impl SdlHostError {
    pub fn message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for SdlHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for SdlHostError {}

pub struct SdlHost {
    config: SdlHostConfig,
}

impl SdlHost {
    pub fn new(config: SdlHostConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, snapshot: SceneSnapshot) -> Result<(), SdlHostError> {
        if let Some(device_environment) = self.config.device_environment() {
            println!("device mode selected; launch with:");
            for (key, value) in device_environment {
                println!("{key}={value}");
            }
        }

        let font_path = self.config.font.resolve()?;
        let sdl = sdl2::init().map_err(SdlHostError::message)?;
        let video = sdl.video().map_err(SdlHostError::message)?;
        let ttf = Sdl2Ttf::load()?;

        let window = video
            .window(
                &self.config.title,
                self.config.window_size.width,
                self.config.window_size.height,
            )
            .position_centered()
            .build()
            .map_err(|error| SdlHostError::message(error.to_string()))?;
        let mut canvas = window
            .into_canvas()
            .present_vsync()
            .build()
            .map_err(|error| SdlHostError::message(error.to_string()))?;
        let texture_creator = canvas.texture_creator();

        let mut host = Host::new();
        host.ingest(snapshot, self.config.window_size)
            .map_err(|error| SdlHostError::message(error.to_string()))?;

        let mut event_pump = sdl.event_pump().map_err(SdlHostError::message)?;
        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        log_intents(host.handle_action(InputAction::B));
                        break 'running;
                    }
                    Event::KeyDown {
                        keycode: Some(keycode),
                        repeat: false,
                        ..
                    } => {
                        if let Some(action) = keyboard_action(keycode) {
                            log_intents(host.handle_action(action));
                        }
                    }
                    _ => {}
                }
            }

            render_scene(
                &mut canvas,
                &texture_creator,
                &font_path,
                &ttf,
                host.scene()
                    .expect("SDL host ingests a scene before rendering")
                    .layout
                    .root
                    .clone(),
                host.focused_key(),
            )?;
            std::thread::sleep(Duration::from_millis(8));
        }

        Ok(())
    }
}

pub fn keyboard_action(keycode: Keycode) -> Option<InputAction> {
    match keycode {
        Keycode::Up => Some(InputAction::DpadUp),
        Keycode::Down => Some(InputAction::DpadDown),
        Keycode::Left => Some(InputAction::DpadLeft),
        Keycode::Right => Some(InputAction::DpadRight),
        Keycode::Space | Keycode::Z => Some(InputAction::A),
        Keycode::Backspace | Keycode::B => Some(InputAction::B),
        Keycode::Return => Some(InputAction::Start),
        Keycode::Tab | Keycode::F1 => Some(InputAction::Select),
        _ => None,
    }
}

fn log_intents(intents: Vec<HostIntent>) {
    for intent in intents {
        println!("intent: {intent:?}");
    }
}

fn render_scene(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font_path: &Path,
    ttf: &Sdl2Ttf,
    root: LayoutNode,
    focused_key: Option<&NodeKey>,
) -> Result<(), SdlHostError> {
    canvas.set_draw_color(to_sdl_color(root.visual.background));
    canvas.clear();
    draw_node(canvas, texture_creator, font_path, ttf, &root, focused_key)?;
    canvas.present();
    Ok(())
}

fn draw_node(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font_path: &Path,
    ttf: &Sdl2Ttf,
    node: &LayoutNode,
    focused_key: Option<&NodeKey>,
) -> Result<(), SdlHostError> {
    let is_focused = focused_key == Some(&node.key);
    let mut background = node.visual.background;
    if is_focused {
        background = node.visual.focused_background.unwrap_or(background);
    }

    if background.a > 0 {
        canvas.set_draw_color(to_sdl_color(background));
        canvas
            .fill_rect(to_sdl_rect(node.rect))
            .map_err(SdlHostError::message)?;
    }

    if let Some(border) = node.visual.border {
        canvas.set_draw_color(to_sdl_color(border));
        canvas
            .draw_rect(to_sdl_rect(node.rect))
            .map_err(SdlHostError::message)?;
    }

    if let Some(text) = &node.text {
        draw_text(
            canvas,
            texture_creator,
            font_path,
            ttf,
            text,
            node.visual.foreground,
            (
                node.rect.x + 12,
                node.rect.y + ((node.rect.height.saturating_sub(18)) / 2) as i32,
            ),
        )?;
    }

    for child in &node.children {
        draw_node(canvas, texture_creator, font_path, ttf, child, focused_key)?;
    }

    Ok(())
}

fn draw_text(
    canvas: &mut Canvas<Window>,
    texture_creator: &TextureCreator<WindowContext>,
    font_path: &Path,
    ttf: &Sdl2Ttf,
    text: &str,
    color: Color,
    position: (i32, i32),
) -> Result<(), SdlHostError> {
    let surface = ttf.render_blended(font_path, 16, text, to_sdl_color(color))?;
    let texture = texture_creator
        .create_texture_from_surface(&surface)
        .map_err(|error| SdlHostError::message(error.to_string()))?;
    let target = SdlRect::new(position.0, position.1, surface.width(), surface.height());
    canvas
        .copy(&texture, None, target)
        .map_err(SdlHostError::message)
}

fn to_sdl_color(color: Color) -> SdlColor {
    SdlColor::RGBA(color.r, color.g, color.b, color.a)
}

fn to_sdl_rect(rect: decklet_core::Rect) -> SdlRect {
    SdlRect::new(rect.x, rect.y, rect.width, rect.height)
}

type TtfInit = unsafe extern "C" fn() -> c_int;
type TtfQuit = unsafe extern "C" fn();
type TtfOpenFont = unsafe extern "C" fn(*const c_char, c_int) -> *mut TtfFontRaw;
type TtfCloseFont = unsafe extern "C" fn(*mut TtfFontRaw);
type TtfRenderUtf8Blended = unsafe extern "C" fn(
    *mut TtfFontRaw,
    *const c_char,
    sdl2::sys::SDL_Color,
) -> *mut sdl2::sys::SDL_Surface;

#[allow(non_camel_case_types)]
enum TtfFontRaw {}

struct Sdl2Ttf {
    _library: Library,
    quit: TtfQuit,
    open_font: TtfOpenFont,
    close_font: TtfCloseFont,
    render_utf8_blended: TtfRenderUtf8Blended,
}

impl Sdl2Ttf {
    fn load() -> Result<Self, SdlHostError> {
        let library = load_ttf_library()?;
        let init = load_symbol::<TtfInit>(&library, b"TTF_Init\0")?;
        let quit = load_symbol::<TtfQuit>(&library, b"TTF_Quit\0")?;
        let open_font = load_symbol::<TtfOpenFont>(&library, b"TTF_OpenFont\0")?;
        let close_font = load_symbol::<TtfCloseFont>(&library, b"TTF_CloseFont\0")?;
        let render_utf8_blended =
            load_symbol::<TtfRenderUtf8Blended>(&library, b"TTF_RenderUTF8_Blended\0")?;

        let init_result = unsafe { init() };
        if init_result != 0 {
            return Err(SdlHostError::message("SDL2_ttf initialization failed"));
        }

        Ok(Self {
            _library: library,
            quit,
            open_font,
            close_font,
            render_utf8_blended,
        })
    }

    fn render_blended(
        &self,
        font_path: &Path,
        size: c_int,
        text: &str,
        color: SdlColor,
    ) -> Result<Surface<'static>, SdlHostError> {
        let path = CString::new(font_path.to_string_lossy().as_bytes()).map_err(|_| {
            SdlHostError::message(format!(
                "font path contains an interior NUL byte: {}",
                font_path.display()
            ))
        })?;
        let text = CString::new(text)
            .map_err(|_| SdlHostError::message("text contains an interior NUL byte"))?;
        let font = unsafe { (self.open_font)(path.as_ptr(), size) };
        if font.is_null() {
            return Err(SdlHostError::message(format!(
                "SDL2_ttf could not open font: {}",
                font_path.display()
            )));
        }

        let font = TtfFont {
            raw: font,
            close_font: self.close_font,
        };
        let surface = unsafe {
            (self.render_utf8_blended)(
                font.raw,
                text.as_ptr(),
                sdl2::sys::SDL_Color {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: color.a,
                },
            )
        };
        if surface.is_null() {
            return Err(SdlHostError::message("SDL2_ttf text rendering failed"));
        }

        Ok(unsafe { Surface::from_ll(surface) })
    }
}

impl Drop for Sdl2Ttf {
    fn drop(&mut self) {
        unsafe { (self.quit)() };
    }
}

struct TtfFont {
    raw: *mut TtfFontRaw,
    close_font: TtfCloseFont,
}

impl Drop for TtfFont {
    fn drop(&mut self) {
        unsafe { (self.close_font)(self.raw) };
    }
}

fn load_ttf_library() -> Result<Library, SdlHostError> {
    const CANDIDATES: &[&str] = &[
        "libSDL2_ttf-2.0.so.0",
        "libSDL2_ttf.so.0",
        "libSDL2_ttf.so",
        "SDL2_ttf.dll",
        "SDL2_ttf",
    ];

    let mut errors = Vec::new();
    for candidate in CANDIDATES {
        match unsafe { Library::new(candidate) } {
            Ok(library) => return Ok(library),
            Err(error) => errors.push(format!("{candidate}: {error}")),
        }
    }

    Err(SdlHostError::message(format!(
        "SDL2_ttf runtime library was not found; tried {}",
        errors.join("; ")
    )))
}

fn load_symbol<T: Copy>(library: &Library, name: &[u8]) -> Result<T, SdlHostError> {
    unsafe {
        library
            .get::<T>(name)
            .map(|symbol| *symbol)
            .map_err(|error| SdlHostError::message(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_maps_to_gamepad_style_actions() {
        assert_eq!(keyboard_action(Keycode::Up), Some(InputAction::DpadUp));
        assert_eq!(keyboard_action(Keycode::Space), Some(InputAction::A));
        assert_eq!(keyboard_action(Keycode::B), Some(InputAction::B));
        assert_eq!(keyboard_action(Keycode::Return), Some(InputAction::Start));
        assert_eq!(keyboard_action(Keycode::Tab), Some(InputAction::Select));
    }

    #[test]
    fn desktop_mode_is_default() {
        let config = SdlHostConfig::desktop();

        assert_eq!(config.mode, LaunchMode::Desktop);
        assert_eq!(config.window_size, DEMO_VIEWPORT);
        assert!(config.device_environment().is_none());
    }

    #[test]
    fn device_mode_exposes_sdl_kmsdrm_environment_without_installing_services() {
        let config = SdlHostConfig {
            mode: LaunchMode::Device,
            ..SdlHostConfig::desktop()
        };

        assert_eq!(config.device_environment(), Some(&DEVICE_SDL_ENVIRONMENT));
    }
}
