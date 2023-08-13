use iced::{theme::{self, ProgressBar}, Theme, widget::{button, button::Appearance, progress_bar}, Color};
use iced_aw::style::colors;


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


pub fn circular_floating_button_style() -> iced::theme::Button {
    theme::Button::Custom(Box::new(
        ButtonStyle::new( theme::Button::Primary, 
                    25.0,
                    Color::from_rgb(0.1, 0.5, 0.9),
                    Color::from_rgb(0.2, 0.6, 1.0),
                    Color::from_rgb(0.0, 0.3, 0.7)
        )
    ))
}

pub fn play_button_style() -> iced::theme::Button {
   theme::Button::Custom(Box::new(ButtonStyle::new(
    theme::Button::Primary,
    5.0,
    Color::from_rgb(0.1, 0.5, 0.9),
    Color::from_rgb(0.2, 0.6, 1.0),
    Color::from_rgb(0.0, 0.3, 0.7),
    )))
} 

pub fn pause_button_style() -> iced::theme::Button {
   theme::Button::Custom(Box::new(ButtonStyle::new(
    theme::Button::Secondary,
    5.0,
    Color::from_rgb(0.5, 0.5, 0.5),   
    Color::from_rgb(0.4, 0.4, 0.4),   
    Color::from_rgb(0.3, 0.3, 0.3),   
)))
} 

pub fn cancel_button_style() -> iced::theme::Button {
   theme::Button::Custom(Box::new(ButtonStyle::new(
    theme::Button::Destructive,
    5.0,
    Color::from_rgb(0.8, 0.2, 0.2),
    Color::from_rgb(0.7, 0.1, 0.1),
    Color::from_rgb(0.6, 0.0, 0.0),
    )))
}


pub fn downloading_pb_style() -> iced::theme::ProgressBar{
   theme::ProgressBar::Custom(Box::new(ProgressBarStyle::new(
    theme::ProgressBar::Primary,
    Color::from_rgb(0.1, 0.5, 0.9),
    )))
}

pub fn paused_pb_style() -> iced::theme::ProgressBar{
   theme::ProgressBar::Custom(Box::new(ProgressBarStyle::new(
    theme::ProgressBar::Primary,
    Color::from_rgb(0.2, 0.2, 0.2),
    )))
}

pub fn done_pb_style() -> iced::theme::ProgressBar{
   theme::ProgressBar::Custom(Box::new(ProgressBarStyle::new(
    theme::ProgressBar::Primary,
    colors::SUCCESS
    )))
}

