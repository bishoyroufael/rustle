mod download_utils;
mod gui;

use gui::rustle_gui::RustleGUI;
use iced::{Settings, window, Application};

fn main() -> iced::Result {

    let font_bytes = include_bytes!("../assets/fonts/victor_mono/static/VictorMono-Medium.ttf");

    let settings = Settings {
        window: window::Settings {
            size: (600, 800),
            resizable: false,
            decorations: true,
            ..window::Settings::default()
        },
        default_font: Some(font_bytes),
        ..Default::default()
    };

    RustleGUI::run(settings)
}
