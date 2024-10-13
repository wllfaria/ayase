use iced::widget::{column, container, horizontal_space, row, text, text_editor};
use iced::{border, Alignment, Border, Element, Length};

use crate::style::{
    button, input, margin_x, margin_y, padding_all, padding_y, BG_ACCENT, BG_PRIMARY, COLOR_BLUE, COLOR_GREEN,
    COLOR_TEXT, FONT_BIG, FONT_BIGGER, RADIUS_SMALL, SPACING_BIG, SPACING_NORMAL, SPACING_SMALL,
};
use crate::{LoadFrom, State};

#[derive(Debug, Clone)]
pub enum Message {
    LoadFromFile,
    LoadFromCode,
    CodeEditor(text_editor::Action),
    LoadAddress(String),
    ConfirmLoad,
}

fn card(inner: Element<'_, Message>) -> Element<'_, Message> {
    container(inner)
        .style(|_| {
            container::Style::default()
                .border(Border::default().rounded(SPACING_NORMAL))
                .background(BG_ACCENT)
        })
        .padding(padding_all(SPACING_BIG))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn load_options<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    row![
        button(
            text("+ FROM FILE").color(COLOR_GREEN).into(),
            Message::LoadFromFile,
            Length::Shrink,
            padding_all(SPACING_NORMAL)
        ),
        margin_x(SPACING_NORMAL),
        button(
            text("+ FROM CODE").color(COLOR_GREEN).into(),
            Message::LoadFromCode,
            Length::Shrink,
            padding_all(SPACING_NORMAL)
        ),
    ]
    .into()
}

fn working_memory<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    column![
        text("Working memory").size(FONT_BIGGER),
        margin_y(SPACING_BIG),
        card(column![text("hi"), text("hi"), text("hi"),].into())
    ]
    .height(Length::Fill)
    .width(Length::Fill)
    .padding(padding_y(SPACING_BIG))
    .into()
}

fn stack_memory<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    column![text("stack memory").size(FONT_BIGGER)]
        .height(Length::Fill)
        .width(Length::Fill)
        .padding(padding_y(SPACING_BIG))
        .into()
}

fn load_details<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    row![
        horizontal_space(),
        column![
            text("Address"),
            margin_y(SPACING_SMALL),
            row![
                container(input("", &state.load_address, Message::LoadAddress, Alignment::Center))
                    .style(|_| container::Style::default()
                        .background(BG_ACCENT)
                        .border(border::rounded(RADIUS_SMALL)))
                    .padding(padding_all(SPACING_NORMAL))
                    .width(65),
                margin_x(SPACING_NORMAL),
                button(
                    text("LOAD").color(COLOR_GREEN).into(),
                    Message::ConfirmLoad,
                    Length::Shrink,
                    padding_all(SPACING_NORMAL)
                )
            ]
        ],
    ]
    .width(Length::Fill)
    .into()
}

fn code_editor<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    column![
        text("Insert assembly code below").color(COLOR_TEXT).size(FONT_BIG),
        margin_y(SPACING_NORMAL),
        text_editor(&state.code_editor)
            .on_action(Message::CodeEditor)
            .height(Length::Fill)
            .style(|_, _| text_editor::Style {
                background: iced::Background::Color(BG_PRIMARY),
                border: Border::default().width(1).color(BG_ACCENT).rounded(SPACING_SMALL),
                icon: BG_ACCENT,
                placeholder: BG_ACCENT,
                value: COLOR_TEXT,
                selection: COLOR_BLUE,
            })
    ]
    .into()
}

fn memory_inspector<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    row![working_memory(state), margin_x(SPACING_BIG), stack_memory(state)].into()
}

pub fn view<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    match state.load_from {
        LoadFrom::None => column![load_options(state), memory_inspector(state)]
            .padding(SPACING_BIG)
            .into(),
        LoadFrom::Code => column![code_editor(state), margin_y(SPACING_BIG), load_details(state)]
            .padding(SPACING_BIG)
            .into(),

        LoadFrom::File => todo!(),
    }
}

pub fn update<const SIZE: usize>(state: &mut State<SIZE>, message: Message) {
    match message {
        Message::LoadFromCode => state.load_from = LoadFrom::Code,
        Message::LoadFromFile => state.load_from = LoadFrom::File,
        Message::CodeEditor(action) => state.code_editor.perform(action),
        Message::LoadAddress(new_address) => {
            if u16::from_str_radix(&new_address, 16).is_ok() || new_address.is_empty() {
                state.load_address = new_address;
                state.load_address.truncate(4);
            }
        }
        Message::ConfirmLoad => {}
    }
}
