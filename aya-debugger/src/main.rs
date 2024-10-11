use iced::{Element, Theme};

mod root;

#[derive(Debug, Default, Clone, Copy)]
enum Screen {
    #[default]
    Root,
}

#[derive(Debug, Default)]
pub struct State {
    screen: Screen,
}

#[derive(Debug, Clone)]
enum Message {
    Root(root::Message),
}

fn main() {
    iced::application("Aya debugger", update, view)
        .window_size((1280., 720.))
        .theme(theme)
        .run()
        .unwrap()
}

fn view(state: &State) -> Element<'_, Message> {
    match state.screen {
        Screen::Root => root::view(state).map(Message::Root),
    }
}

fn update(state: &mut State, message: Message) {
    match message {
        Message::Root(message) => root::update(state, message),
    }
}

fn theme(_: &State) -> Theme {
    Theme::KanagawaDragon
}
