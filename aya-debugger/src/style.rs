use iced::widget::{button as iced_button, container, text_input, Space};
use iced::{border, color, Alignment, Border, Color, Element, Length, Padding};

pub static FONT: &str = "JetBrainsMono Nerd Font";

pub static FONT_BIGGER: u16 = 24;
pub static FONT_BIG: u16 = 20;
pub static FONT_REGULAR: u16 = 16;
pub static FONT_SMALL: u16 = 12;

pub static SPACING_NORMAL: u16 = 10;
pub static SPACING_BIG: u16 = 20;
pub static SPACING_SMALL: u16 = 5;

pub static RADIUS_SMALL: u16 = 8;

pub static BG_PRIMARY: Color = color!(0x181616, 1.0);
pub static BG_SECONDARY: Color = color!(0x282727, 1.0);
pub static BG_ACCENT: Color = color!(0x393836, 1.0);
pub static COLOR_TEXT: Color = color!(0xC5C9C5, 1.0);
pub static COLOR_TEXT_MUTED: Color = color!(0x737C73, 1.0);
pub static COLOR_PURPLE: Color = color!(0xC474CE, 1.0);
pub static COLOR_GREEN: Color = color!(0x87A987, 1.0);
pub static COLOR_ORANGE: Color = color!(0xC4B28A, 1.0);
pub static COLOR_BLUE: Color = color!(0x8BA4B0, 1.0);

pub fn margin(horizontal: u16, vertical: u16) -> Space {
    Space::new(horizontal, vertical)
}

pub fn margin_y(amount: u16) -> Space {
    margin(0, amount)
}

pub fn margin_x(amount: u16) -> Space {
    margin(amount, 0)
}

pub fn padding(top: u16, right: u16, bottom: u16, left: u16) -> Padding {
    Padding::default().top(top).right(right).bottom(bottom).left(left)
}

pub fn padding_x(amount: u16) -> Padding {
    padding(0, amount, 0, amount)
}

pub fn padding_y(amount: u16) -> Padding {
    padding(amount, 0, amount, 0)
}

pub fn padding_all(amount: u16) -> Padding {
    padding(amount, amount, amount, amount)
}

pub fn button<'a, T: Clone + 'a>(
    inner: Element<'a, T>,
    on_press: T,
    width: impl Into<Length>,
    padding: Padding,
) -> Element<'a, T> {
    iced_button(
        container(inner)
            .style(|_| {
                container::Style::default()
                    .background(BG_ACCENT)
                    .border(border::rounded(RADIUS_SMALL))
            })
            .padding(padding)
            .align_x(Alignment::Center)
            .width(width),
    )
    .style(|_, _| iced_button::Style::default())
    .padding(0)
    .on_press(on_press)
    .into()
}

pub fn input<'a, S: AsRef<str>, T: Clone + 'a, K: Fn(String) -> T + 'a>(
    placeholder: S,
    value: S,
    on_input: K,
    align: Alignment,
) -> Element<'a, T> {
    text_input(placeholder.as_ref(), value.as_ref())
        .on_input(on_input)
        .padding(0)
        .align_x(align)
        .style(|_, _| text_input::Style {
            background: iced::Background::Color(BG_ACCENT),
            border: Border::default(),
            icon: BG_ACCENT,
            placeholder: BG_ACCENT,
            selection: COLOR_BLUE,
            value: COLOR_TEXT_MUTED,
        })
        .into()
}
