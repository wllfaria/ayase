mod container;
mod sidebar;
mod style;

use std::fmt;

use aya_console::memory::LinearMemory;
use aya_core::cpu::Cpu;
use aya_core::MEMORY_SIZE;
use iced::widget::{row, text_editor};
use iced::{Element, Font, Theme};
use style::FONT;

#[derive(Debug, Default, Clone, Copy)]
pub enum CpuStatus {
    #[default]
    Paused,
    Running,
}
impl fmt::Display for CpuStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CpuStatus::Paused => f.write_str("PAUSED"),
            CpuStatus::Running => f.write_str("RUNNING"),
        }
    }
}

#[derive(Debug, Default)]
pub enum LoadFrom {
    #[default]
    None,
    Code,
}

#[derive(Debug)]
pub struct State<const SIZE: usize> {
    pub cpu_status: CpuStatus,
    pub cpu: Cpu<SIZE, LinearMemory<SIZE>>,
    pub instruction_delay: usize,
    pub load_from: LoadFrom,
    pub code_editor: text_editor::Content,
    pub load_address: String,
    pub working_mem: u16,
    pub stack_mem: u16,
}

impl<const SIZE: usize> Default for State<SIZE> {
    fn default() -> Self {
        Self {
            cpu_status: Default::default(),
            instruction_delay: 100,
            load_from: LoadFrom::None,
            code_editor: text_editor::Content::default(),
            load_address: String::from("0000"),
            working_mem: 0,
            stack_mem: 256,
            cpu: Cpu::new(LinearMemory::default()),
        }
    }
}

impl<const SIZE: usize> State<SIZE> {
    pub fn fetch_register(&self, register: &str) -> u16 {
        self.cpu.registers.fetch(register.try_into().unwrap())
    }
}

#[derive(Debug, Clone)]
enum Message {
    Container(container::Message),
    Sidebar(sidebar::Message),
}

fn main() {
    iced::application("Aya debugger", update::<MEMORY_SIZE>, view)
        .font(FONT.as_bytes())
        .default_font(Font::with_name(FONT))
        .window_size((1280., 720.))
        .resizable(false)
        .theme(theme)
        .run()
        .unwrap()
}

fn view<const SIZE: usize>(state: &State<SIZE>) -> Element<'_, Message> {
    row![
        sidebar::view(state).map(Message::Sidebar),
        container::view(state).map(Message::Container)
    ]
    .into()
}

fn update<const SIZE: usize>(state: &mut State<SIZE>, message: Message) {
    match message {
        Message::Container(message) => container::update(state, message),
        Message::Sidebar(message) => sidebar::update(state, message),
    }
}

fn theme<const SIZE: usize>(_: &State<SIZE>) -> Theme {
    Theme::KanagawaDragon
}
