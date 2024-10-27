mod color;
mod consts;
pub mod decoder;
mod error;

pub use color::Color;
pub use decoder::decode;
use error::{Error, Result};

#[derive(Debug)]
pub struct Bitmap {
    header: BitmapHeader,
    info_header: BitmapInfoHeader,
    pub palette: Vec<Color>,
    pub data: Vec<Color>,
}

#[derive(Debug)]
struct BitmapHeader {
    file_size: u32,
    data_offset: u32,
}

#[derive(Debug)]
struct BitmapInfoHeader {
    width: u32,
    height: u32,
    bit_depth: BitDepth,
    num_colors: u32,
    image_size: u32,
    important_colors: u32,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BitDepth {
    MonoChrome,
    Bit4,
    Bit8,
    Bit16,
    Bit24,
}

impl BitDepth {
    pub(crate) fn has_palette(&self) -> bool {
        matches!(self, BitDepth::MonoChrome | BitDepth::Bit4 | BitDepth::Bit8)
    }
}

impl TryFrom<u16> for BitDepth {
    type Error = Error;

    fn try_from(depth: u16) -> Result<Self> {
        match depth {
            1 => Ok(Self::MonoChrome),
            4 => Ok(Self::Bit4),
            8 => Ok(Self::Bit8),
            16 => Ok(Self::Bit16),
            24 => Ok(Self::Bit24),
            _ => Err(Error::NonBitmap),
        }
    }
}
