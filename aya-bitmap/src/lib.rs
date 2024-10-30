mod color;
mod consts;
pub mod decoder;
mod error;

pub use color::Color;
pub use decoder::decode;
use error::{Error, Result};

#[derive(Debug)]
pub struct Bitmap {
    file_name: String,
    header: BitmapHeader,
    info_header: BitmapInfoHeader,
    palette: Vec<Color>,
    data: Vec<Color>,
}

impl Bitmap {
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn data(&self) -> &[Color] {
        &self.data
    }

    pub fn header(&self) -> &BitmapHeader {
        &self.header
    }

    pub fn info_header(&self) -> &BitmapInfoHeader {
        &self.info_header
    }

    pub fn palette(&self) -> &[Color] {
        &self.palette
    }
}

#[derive(Debug)]
pub struct BitmapHeader {
    file_size: u32,
    data_offset: u32,
}

impl BitmapHeader {
    pub fn file_size(&self) -> u32 {
        self.file_size
    }

    pub fn data_offset(&self) -> u32 {
        self.data_offset
    }
}

#[derive(Debug)]
pub struct BitmapInfoHeader {
    width: u32,
    height: u32,
    bit_depth: BitDepth,
    num_colors: u32,
    image_size: u32,
    important_colors: u32,
}

impl BitmapInfoHeader {
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn image_size(&self) -> u32 {
        self.image_size
    }

    pub fn important_colors(&self) -> u32 {
        self.important_colors
    }
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
