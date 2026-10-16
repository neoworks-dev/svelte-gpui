//! What the app and the window tell each other about the window itself: the options the
//! app opens it with, the requests it makes of it afterwards, and the state the host
//! reports back.

use serde_json::Value;

/// How the app wants its window to open, from `start(App, props, { window })`. Anything
/// the app leaves out stays as the host's default, and the host's own flags override these.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WindowSettings {
    pub title: Option<String>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,
}

impl WindowSettings {
    /// Reads the object the app passed; fields of the wrong type are left out.
    pub fn from_json(value: &Value) -> Self {
        let number = |name: &str| value[name].as_f64().filter(|found| *found > 0.0).map(|found| found as f32);
        WindowSettings {
            title: value["title"].as_str().map(str::to_string),
            width: number("width"),
            height: number("height"),
            min_width: number("minWidth"),
            min_height: number("minHeight"),
        }
    }
}

/// Something the app asked of its window. The host carries it out on its next tick,
/// because it needs the window itself.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowCommand {
    Title(String),
    Minimize,
    Maximize,
    Unmaximize,
    ToggleMaximize,
    SetFullscreen(bool),
    ToggleFullscreen,
    /// Closes the window without asking the app's close handlers.
    Destroy,
}

impl WindowCommand {
    /// Reads the operation the app sent through `host.window`. `None` for one it does not know.
    pub fn parse(operation: &str, arguments: &Value) -> Option<Self> {
        Some(match operation {
            "title" => WindowCommand::Title(arguments["title"].as_str()?.to_string()),
            "minimize" => WindowCommand::Minimize,
            "maximize" => WindowCommand::Maximize,
            "unmaximize" => WindowCommand::Unmaximize,
            "toggleMaximize" => WindowCommand::ToggleMaximize,
            "fullscreen" => WindowCommand::SetFullscreen(arguments["on"].as_bool()?),
            "toggleFullscreen" => WindowCommand::ToggleFullscreen,
            "destroy" => WindowCommand::Destroy,
            _ => return None,
        })
    }
}

/// What the app can read about its window, as the host last reported it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowState {
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub focused: bool,
    pub maximized: bool,
    pub fullscreen: bool,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn settings_ignore_the_wrong_types() {
        let settings =
            WindowSettings::from_json(&json!({ "title": "A", "width": 800, "height": "tall", "minWidth": -1 }));
        assert_eq!(settings.title.as_deref(), Some("A"));
        assert_eq!(settings.width, Some(800.0));
        assert_eq!(settings.height, None);
        assert_eq!(settings.min_width, None);
    }

    #[test]
    fn commands_need_their_arguments() {
        assert_eq!(WindowCommand::parse("title", &json!({})), None);
        assert_eq!(
            WindowCommand::parse("fullscreen", &json!({ "on": true })),
            Some(WindowCommand::SetFullscreen(true))
        );
        assert_eq!(WindowCommand::parse("nonsense", &Value::Null), None);
    }
}
