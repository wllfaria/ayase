use iced::widget::text;
use iced::Element;

use crate::State;

#[derive(Debug, Clone)]
pub enum Message {}

pub fn view(_state: &State) -> Element<'_, Message> {
    text("from root").into()
}

pub fn update(_state: &mut State, _message: Message) {}
