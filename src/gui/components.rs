use iced::{Element, Renderer};
use iced::widget::{Text, ProgressBar, Button};
use iced_aw::{Badge, style::BadgeStyles, Icon, ICON_FONT};
use super::rustle_gui::Message;

/// Returns a `Text` widget displaying a plus icon.
pub fn plus_icon() -> Text<'static> {
    Text::new(Icon::Plus.to_string()).font(ICON_FONT).into()
}

/// Returns a `Text` widget displaying an info icon.
pub fn info_icon() -> Text<'static> {
    Text::new(Icon::Info.to_string()).font(ICON_FONT).into()
}

/// Returns a `Text` widget displaying a play icon.
pub fn play_icon() -> Text<'static> {
    Text::new(Icon::Play.to_string()).font(ICON_FONT).into()
}

/// Returns a `Text` widget displaying a pause icon.
pub fn pause_icon() -> Text<'static> {
    Text::new(Icon::Pause.to_string()).font(ICON_FONT)
}

/// Returns a `Text` widget displaying a cancel icon.
pub fn cancel_icon() -> Text<'static> {
    Text::new(Icon::X.to_string()).font(ICON_FONT)
}

/// Creates a `Badge` element with the specified text and style.
///
/// # Arguments
///
/// * `text` - The text content of the badge.
/// * `style` - The style to apply to the badge.
///
/// # Returns
///
/// Returns an `Element` containing the badge widget.
pub fn badge(text: String, style: BadgeStyles) -> Element<'static, Message> {
    Badge::new(Text::new(text)).style(style).into()
}

/// Creates a `ProgressBar` widget with the specified value and style.
///
/// # Arguments
///
/// * `value` - The current value of the progress bar.
/// * `style` - The style to apply to the progress bar.
///
/// # Returns
///
/// Returns a `ProgressBar` widget.
pub fn progress_bar(value: f32, style: iced::theme::ProgressBar) -> iced::widget::ProgressBar<Renderer> {
    ProgressBar::new(0.0..=100.0, value).style(style).into()
}

/// Creates a `Button` widget with the provided text component and optional message callback.
///
/// # Arguments
///
/// * `text_component` - The text to display on the button.
/// * `on_message` - An optional message to send when the button is pressed.
/// * `style` - The style to apply to the button.
///
/// # Returns
///
/// Returns a `Button` widget.
pub fn button(text_component: Text, on_message: Option<Message>, style: iced::theme::Button) -> iced::widget::Button<Message> { 
    match on_message {
        Some(callback_message) => {
            Button::new(text_component).on_press(callback_message).style(style)
        },
        None => {
            Button::new(text_component).style(style)
        },
    }
}