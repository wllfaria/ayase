mod error;
mod raylib;

use aya_cpu::memory::Addressable;
use error::Result;
pub use raylib::RaylibRenderer;

pub trait Renderer {
    fn should_close(&self) -> bool;
    fn should_draw(&self) -> bool;
    fn draw_frame(&mut self, memory: &mut impl Addressable) -> Result<()>;
}
