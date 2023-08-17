use iced::{theme::{self}, Theme, widget::{button, button::Appearance, progress_bar, container}, Color};
use iced_aw::style::colors;

pub const GREEN_COLOR_MAIN : Color = Color::from_rgb(0.0, 0.749, 0.388);
pub const BLUE_COLOR_MAIN : Color = Color::from_rgb(0.1, 0.5, 0.9);

struct ContainerStyle {
    theme: theme::Container,
    bg_color : Color
}

impl ContainerStyle{
    pub fn new(theme: theme::Container, bg_color: Color) -> Self {
        Self { theme, bg_color }
    }
}

impl container::StyleSheet for ContainerStyle{
    type Style = Theme;
    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let mut appearance = style.appearance(&self.theme);
        appearance.background = self.bg_color.into();

        appearance
    }
}


struct ProgressBarStyle {
    theme: theme::ProgressBar,
    bg_color : Color
}

impl ProgressBarStyle {
    pub fn new(theme: theme::ProgressBar, bg_color: Color) -> Self {
        Self { theme, bg_color }
    }
}

impl progress_bar::StyleSheet for ProgressBarStyle {
    type Style = Theme;
    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        let mut appearance = style.appearance(&self.theme);
        appearance.bar = self.bg_color.into();

        appearance
    }
}

/* 
    Button Style 
*/

struct ButtonStyle {
    theme: theme::Button,
    border_radius : f32,
    normal_bg_color: Color,
    hovered_bg_color: Color,
    clicked_bg_color: Color,
}

impl ButtonStyle {
    pub fn new(theme: theme::Button, 
                border_radius:f32,
                normal_bg_color: Color,
                hovered_bg_color: Color,
                clicked_bg_color: Color
            ) -> Self {
        Self {
            theme,
            border_radius,
            normal_bg_color,
            hovered_bg_color,
            clicked_bg_color,
        }
    }
}

impl button::StyleSheet for ButtonStyle {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> Appearance {
        let mut appearance = style.active(&self.theme);
        appearance.border_radius = self.border_radius;
        appearance.background = self.normal_bg_color.into(); //

        appearance
    }

    fn hovered(&self, style: &Self::Style) -> Appearance {
        let mut appearance = style.hovered(&self.theme); // Use hovered style
        appearance.border_radius = self.border_radius; 
        appearance.background = self.hovered_bg_color.into();//

        appearance
    }

    fn pressed(&self, style: &Self::Style) -> Appearance {
        let mut appearance = style.pressed(&self.theme); // Use pressed style
        appearance.border_radius = self.border_radius; 
        appearance.background = self.clicked_bg_color.into(); //

        appearance
    }

}


/// Returns a custom button style for circular floating buttons.
///
/// # Returns
///
/// Returns a button style with a circular shape, using main green color for background and gradients for hover effect.
pub fn circular_floating_button_style() -> iced::theme::Button {
    theme::Button::Custom(Box::new(
        ButtonStyle::new( theme::Button::Primary, 
                    100.0,
                    GREEN_COLOR_MAIN,
                    Color::from_rgb(0.0, 0.85, 0.35),
                    Color::from_rgb(0.0, 0.7, 0.35)
        )
    ))
}

/// Returns a custom button style for play/submit buttons.
///
/// # Returns
///
/// Returns a button style with a slightly rounded rectangle shape, using main blue color for background and gradients for hover effect.
pub fn play_submit_button_style() -> iced::theme::Button {
   theme::Button::Custom(Box::new(ButtonStyle::new(
    theme::Button::Primary,
    5.0,
    BLUE_COLOR_MAIN,
    Color::from_rgb(0.2, 0.6, 1.0),
    Color::from_rgb(0.0, 0.3, 0.7),
    )))
} 

/// Returns a custom button style for pause buttons.
///
/// # Returns
///
/// Returns a button style with a slightly rounded rectangle shape, using a gray color for background and darker shades for hover effect.
pub fn pause_button_style() -> iced::theme::Button {
   theme::Button::Custom(Box::new(ButtonStyle::new(
    theme::Button::Secondary,
    5.0,
    Color::from_rgb(0.5, 0.5, 0.5),   
    Color::from_rgb(0.4, 0.4, 0.4),   
    Color::from_rgb(0.3, 0.3, 0.3),   
)))
} 

/// Returns a custom button style for cancel buttons.
///
/// # Returns
///
/// Returns a button style with a slightly rounded rectangle shape, using a red color for background and darker shades for hover effect.
pub fn cancel_button_style() -> iced::theme::Button {
   theme::Button::Custom(Box::new(ButtonStyle::new(
    theme::Button::Destructive,
    5.0,
    Color::from_rgb(0.8, 0.2, 0.2),
    Color::from_rgb(0.7, 0.1, 0.1),
    Color::from_rgb(0.6, 0.0, 0.0),
    )))
}

/// Returns a custom progress bar style for downloading state.
///
/// # Returns
///
/// Returns a progress bar style with the primary color transitioning from light to dark blue.
pub fn downloading_pb_style() -> iced::theme::ProgressBar{
   theme::ProgressBar::Custom(Box::new(ProgressBarStyle::new(
    theme::ProgressBar::Primary,
    Color::from_rgb(0.1, 0.5, 0.9),
    )))
}

/// Returns a custom progress bar style for paused state.
///
/// # Returns
///
/// Returns a progress bar style with the primary color set to gray.
pub fn paused_pb_style() -> iced::theme::ProgressBar{
   theme::ProgressBar::Custom(Box::new(ProgressBarStyle::new(
    theme::ProgressBar::Primary,
    Color::from_rgb(0.2, 0.2, 0.2),
    )))
}

/// Returns a custom progress bar style for done state.
///
/// # Returns
///
/// Returns a progress bar style with the primary color set to a success color.
pub fn done_pb_style() -> iced::theme::ProgressBar{
   theme::ProgressBar::Custom(Box::new(ProgressBarStyle::new(
    theme::ProgressBar::Primary,
    colors::SUCCESS
    )))
}

/// Returns a custom container style with a white background.
///
/// # Returns
///
/// Returns a container style with a white background color.
pub fn white_container_style() -> iced::theme::Container{
   theme::Container::Custom(Box::new(ContainerStyle::new(
    theme::Container::Box,
    Color::WHITE
    )))
}

/// Returns a custom text style with a gray color and reduced opacity.
///
/// # Returns
///
/// Returns a text style with a gray color and reduced opacity for a subdued appearance.
pub fn grey_color_text_style() -> theme::Text { 
    theme::Text::Color(
        Color::from_rgba(0.5, 0.5, 0.5, 0.6)
    )
}

