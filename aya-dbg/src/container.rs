use aya_cpu::memory::Addressable;
use iced::widget::{column, container, horizontal_space, row, text, text_editor, Column, Row};
use iced::{border, Alignment, Border, Element, Length};

use crate::style::{
    button, input, margin_x, margin_y, padding_all, padding_y, BG_ACCENT, BG_PRIMARY, BG_SECONDARY, COLOR_BLUE,
    COLOR_GREEN, COLOR_PURPLE, COLOR_TEXT, FONT_BIG, FONT_BIGGER, RADIUS_SMALL, SPACING_BIG, SPACING_NORMAL,
    SPACING_SMALL,
};
use crate::{LoadFrom, State};

const STACK_PAGE: u16 = 255;

#[derive(Debug, Clone)]
pub enum Message {
    LoadFromCode,
    CodeEditor(text_editor::Action),
    LoadAddress(String),
    ConfirmLoad,
    CancelLoad,
    WorkingMemNext,
    WorkingMemPrev,
    StackMemNext,
    StackMemPrev,
}

fn card(inner: Element<'_, Message>) -> Element<'_, Message> {
    container(inner)
        .style(|_| {
            container::Style::default()
                .border(Border::default().rounded(SPACING_NORMAL))
                .background(BG_SECONDARY)
        })
        .padding(padding_all(SPACING_BIG))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn load_options<'a>() -> Element<'a, Message> {
    row![button(
        text("+ FROM CODE").color(COLOR_GREEN).into(),
        Message::LoadFromCode,
        Length::Shrink,
        padding_all(SPACING_NORMAL)
    ),]
    .into()
}

#[derive(Debug)]
enum MemorySection {
    Stack,
    Working,
}

fn memory_rows<'a>(state: &State, section: MemorySection) -> Element<'a, Message> {
    let mut lines: Vec<Vec<Element<'a, Message>>> = vec![];

    let start = match section {
        MemorySection::Stack => state.stack_mem * STACK_PAGE - 1,
        MemorySection::Working => state.working_mem * 16,
    };

    let size = 8;
    for i in 0..16 {
        let start = start + i * size * 2;

        let mem = state.cpu.memory.inspect_address(start, size as usize).unwrap();
        let mut line: Vec<Element<'a, Message>> = vec![];

        line.push(text(format!("{:04X}", start)).into());

        mem.into_iter().for_each(|val| {
            line.push(margin_x(SPACING_NORMAL).into());
            line.push(text(format!("{:04X}", val)).into());
        });
        lines.push(line);
    }

    let mut rows: Vec<Element<Message>> = vec![];
    for line in lines {
        let row = Element::from(Row::from_vec(line));
        rows.push(row);
        rows.push(margin_y(SPACING_SMALL).into());
    }
    Column::from_vec(rows).into()
}

fn working_memory(state: &State) -> Element<'_, Message> {
    column![
        text("Working memory").size(FONT_BIGGER),
        margin_y(SPACING_BIG),
        card(
            column![
                row![
                    container(row![
                        text("Starting position:").color(COLOR_TEXT),
                        text(format!("{:04X}", state.working_mem * 16)).color(COLOR_TEXT)
                    ])
                    .padding(padding_all(SPACING_NORMAL))
                    .style(|_| container::Style::default()
                        .background(BG_ACCENT)
                        .border(border::rounded(SPACING_NORMAL))),
                    horizontal_space(),
                    button(
                        text("< PREV").color(COLOR_BLUE).into(),
                        Message::WorkingMemPrev,
                        Length::Shrink,
                        padding_all(SPACING_NORMAL)
                    ),
                    margin_x(SPACING_NORMAL),
                    button(
                        text("NEXT> ").color(COLOR_BLUE).into(),
                        Message::WorkingMemNext,
                        Length::Shrink,
                        padding_all(SPACING_NORMAL)
                    )
                ],
                margin_y(SPACING_BIG),
                memory_rows(state, MemorySection::Working),
            ]
            .into()
        )
    ]
    .height(Length::Fill)
    .width(Length::Fill)
    .padding(padding_y(SPACING_BIG))
    .into()
}

fn stack_memory(state: &State) -> Element<'_, Message> {
    let start = state.stack_mem * STACK_PAGE - 1;
    column![
        text("Stack memory").size(FONT_BIGGER),
        margin_y(SPACING_BIG),
        card(
            column![
                row![
                    container(row![
                        text("Starting position:").color(COLOR_TEXT),
                        text(format!("{:04X}", start)).color(COLOR_TEXT)
                    ])
                    .padding(padding_all(SPACING_NORMAL))
                    .style(|_| container::Style::default()
                        .background(BG_ACCENT)
                        .border(border::rounded(SPACING_NORMAL))),
                    horizontal_space(),
                    button(
                        text("< PREV").color(COLOR_BLUE).into(),
                        Message::StackMemPrev,
                        Length::Shrink,
                        padding_all(SPACING_NORMAL)
                    ),
                    margin_x(SPACING_NORMAL),
                    button(
                        text("NEXT> ").color(COLOR_BLUE).into(),
                        Message::StackMemNext,
                        Length::Shrink,
                        padding_all(SPACING_NORMAL)
                    )
                ],
                margin_y(SPACING_BIG),
                memory_rows(state, MemorySection::Stack),
            ]
            .into()
        )
    ]
    .height(Length::Fill)
    .width(Length::Fill)
    .padding(padding_y(SPACING_BIG))
    .into()
}

fn load_details(state: &State) -> Element<'_, Message> {
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
                    text("CANCEL").color(COLOR_PURPLE).into(),
                    Message::CancelLoad,
                    Length::Shrink,
                    padding_all(SPACING_NORMAL)
                ),
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

fn code_editor(state: &State) -> Element<'_, Message> {
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

fn memory_inspector(state: &State) -> Element<'_, Message> {
    row![working_memory(state), margin_x(SPACING_BIG), stack_memory(state)].into()
}

pub fn view(state: &State) -> Element<'_, Message> {
    match state.load_from {
        LoadFrom::None => column![load_options(), memory_inspector(state)]
            .padding(SPACING_BIG)
            .into(),
        LoadFrom::Code => column![code_editor(state), margin_y(SPACING_BIG), load_details(state)]
            .padding(SPACING_BIG)
            .into(),
    }
}

pub fn update(state: &mut State, message: Message) {
    match message {
        Message::WorkingMemPrev => state.working_mem = state.working_mem.saturating_sub(1),
        Message::WorkingMemNext => state.working_mem = u16::min(u16::MAX / 16, state.working_mem + 1),
        Message::StackMemPrev => state.stack_mem = state.stack_mem.saturating_sub(1),
        Message::StackMemNext => state.stack_mem = u16::min(256, state.stack_mem + 1),
        Message::LoadFromCode => state.load_from = LoadFrom::Code,
        Message::CodeEditor(action) => state.code_editor.perform(action),
        Message::LoadAddress(new_address) => {
            if u16::from_str_radix(&new_address, 16).is_ok() || new_address.is_empty() {
                state.load_address = new_address;
                state.load_address.truncate(4);
            }
        }
        Message::ConfirmLoad => {
            let bytecode = aya_compiler::compile(state.code_editor.text());
            let address = u16::from_str_radix(&state.load_address, 16).unwrap_or(0x0000);
            state.cpu.load_into_address(bytecode, address).unwrap();
            state.load_from = LoadFrom::None;
        }
        Message::CancelLoad => state.load_from = LoadFrom::None,
    }
}
