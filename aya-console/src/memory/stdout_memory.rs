use aya_cpu::memory::Addressable;
use aya_cpu::word::Word;

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
pub struct OutputMemory {}
fn clear_screen() {
    print!("\x1b[2J");
}
fn move_cursor(x: u16, y: u16) {
    print!("\x1b[{y};{x}H");
}
fn write_char(ch: char) {
    print!("{ch}");
}
impl Addressable for OutputMemory {
    fn write(&mut self, _: Word, _: u8) -> aya_cpu::memory::Result<()> {
        Ok(())
    }

    fn write_word(&mut self, address: Word, word: u16) -> aya_cpu::memory::Result<()> {
        let [ch, command] = word.to_le_bytes();
        match command.into() {
            OutputCommand::ClearScreen => clear_screen(),
            OutputCommand::None => (),
        };
        let x = (u16::from(address) % 50) + 1;
        let y = (u16::from(address) / 50) + 1;
        move_cursor(x, y);
        write_char(ch as char);
        Ok(())
    }

    fn read(&self, _: Word) -> aya_cpu::memory::Result<u8> {
        Ok(0)
    }
}
