use iced::widget::{column, container, horizontal_space, row, text, text_input};
use iced::{border, Alignment, Border, Color, Element, Length};

use crate::style::{
    button, input, margin_x, margin_y, padding_all, padding_x, padding_y, BG_ACCENT, BG_SECONDARY, COLOR_BLUE,
    COLOR_GREEN, COLOR_ORANGE, COLOR_PURPLE, COLOR_TEXT, COLOR_TEXT_MUTED, FONT_BIG, FONT_REGULAR, FONT_SMALL,
    RADIUS_SMALL, SPACING_BIG, SPACING_NORMAL, SPACING_SMALL,
};
use crate::State;

#[derive(Debug, Clone)]
pub enum Message {
    Step,
    Run,
    DelayChanged(String),
}

fn cpu_status<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    column![
        text("Status").color(COLOR_TEXT).size(FONT_BIG),
        margin_y(SPACING_NORMAL),
        container(text(state.cpu_status.to_string()).color(COLOR_ORANGE))
            .style(|_| container::Style::default()
                .background(BG_ACCENT)
                .border(border::rounded(RADIUS_SMALL)))
            .padding(padding_y(SPACING_SMALL))
            .align_x(Alignment::Center)
            .width(Length::Fill)
    ]
    .into()
}

fn top_section<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    column![
        cpu_status(state),
        margin_y(SPACING_NORMAL),
        row![
            button(
                text("STEP").color(COLOR_BLUE).into(),
                Message::Step,
                Length::Fill,
                padding_y(SPACING_SMALL)
            ),
            margin_x(SPACING_NORMAL),
            button(
                text("RUN").color(COLOR_GREEN).into(),
                Message::Run,
                Length::Fill,
                padding_y(SPACING_SMALL)
            ),
        ],
        margin_y(SPACING_NORMAL),
        container(row![
            text("DELAY").color(COLOR_TEXT_MUTED),
            horizontal_space(),
            input(
                "",
                &state.instruction_delay.to_string(),
                Message::DelayChanged,
                Alignment::End
            ),
            container(text(" MS").size(FONT_SMALL).color(COLOR_TEXT_MUTED))
                .align_y(Alignment::Center)
                .height(22),
        ])
        .style(|_| container::Style::default()
            .background(BG_ACCENT)
            .border(border::rounded(RADIUS_SMALL)))
        .padding(padding_all(SPACING_SMALL))
        .width(Length::Fill),
    ]
    .padding(padding_x(SPACING_BIG))
    .into()
}

fn register<'a, const SIZE: usize>(state: &State<SIZE>, name: &'a str, color: Option<Color>) -> Element<'a, Message> {
    let name_color = color.unwrap_or(COLOR_TEXT_MUTED);
    let value_color = color.unwrap_or(COLOR_TEXT);
    let address = state.fetch_register(name);

    container(row![
        text(name).size(FONT_REGULAR).color(name_color),
        horizontal_space(),
        text("0x").size(FONT_REGULAR).color(value_color),
        text(format!("{address:04X}")).size(FONT_REGULAR).color(value_color),
    ])
    .style(|_| {
        container::Style::default()
            .background(BG_ACCENT)
            .border(border::rounded(RADIUS_SMALL))
    })
    .padding(padding_all(SPACING_SMALL))
    .width(Length::Fill)
    .into()
}

fn register_section<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    column![
        text("Registers").color(COLOR_TEXT).size(FONT_BIG),
        margin_y(10),
        register(state, "RET", None),
        margin_y(5),
        register(state, "IP", Some(COLOR_BLUE)),
        margin_y(5),
        register(state, "R1", None),
        margin_y(5),
        register(state, "R2", None),
        margin_y(5),
        register(state, "R3", None),
        margin_y(5),
        register(state, "R4", None),
        margin_y(5),
        register(state, "R5", None),
        margin_y(5),
        register(state, "R6", None),
        margin_y(5),
        register(state, "R7", None),
        margin_y(5),
        register(state, "R8", None),
        margin_y(5),
        register(state, "SP", Some(COLOR_ORANGE)),
        margin_y(5),
        register(state, "FP", Some(COLOR_PURPLE)),
    ]
    .padding(padding_x(SPACING_BIG))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

pub fn view<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    container(column![
        top_section(state),
        margin_y(SPACING_BIG),
        container(margin_y(1).height(1).width(Length::Fill)).style(|_| COLOR_TEXT_MUTED.into()),
        margin_y(SPACING_BIG),
        register_section(state),
    ])
    .style(|_| BG_SECONDARY.into())
    .padding(padding_y(SPACING_BIG))
    .width(250)
    .height(Length::Fill)
    .into()
}

pub fn update<const SIZE: usize>(state: &mut State<SIZE>, message: Message) {
    match message {
        Message::Step => _ = state.cpu.step(&mut |_, _| {}),
        Message::DelayChanged(new_delay) => match new_delay.parse::<usize>() {
            Ok(new_delay) => state.instruction_delay = usize::min(new_delay, 5000),
            Err(_) if new_delay.is_empty() => state.instruction_delay = 0,
            Err(_) => {}
        },
        // TODO: make cpu run and respect instruction delay
        Message::Run => {}
    }
}
