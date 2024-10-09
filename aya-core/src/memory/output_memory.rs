use super::Addressable;
use crate::word::Word;

#[repr(u8)]
enum OutputCommand {
    None,
    ClearScreen,
}

impl From<u8> for OutputCommand {
    fn from(value: u8) -> Self {
        match value {
            0xff => OutputCommand::ClearScreen,
            _ => OutputCommand::None,
        }
    }
}

#[derive(Debug, Default)]
pub struct OutputMemory<const SIZE: usize> {}

fn clear_screen() {
    print!("\x1b[2J");
}

fn move_cursor(x: u16, y: u16) {
    print!("\x1b[{y};{x}H");
}

fn write_char(ch: char) {
    print!("{ch}");
}

impl<const SIZE: usize> Addressable<SIZE> for OutputMemory<SIZE> {
    fn write(&mut self, _: Word<SIZE>, _: u8) -> super::Result<SIZE, ()> {
        Ok(())
    }

    fn write_word(&mut self, address: Word<SIZE>, word: u16) -> super::Result<SIZE, ()> {
        let ch = (word & 0x00ff) as u8;
        let command = ((word & 0xff00) >> 8) as u8;

        match command.into() {
            OutputCommand::ClearScreen => clear_screen(),
            OutputCommand::None => (),
        };

        let x = (u16::from(address) % 16) + 1;
        let y = (u16::from(address) / 16) + 1;
        move_cursor(x, y);
        write_char(ch as char);

        Ok(())
    }

    fn read(&mut self, _: Word<SIZE>) -> super::Result<SIZE, u8> {
        Ok(0)
    }
}
