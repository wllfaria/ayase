use std::path::Path;

use super::color::Color;
use super::consts::{HEADER_SIZE, INFO_HEADER_SIZE};
use super::error::{Error, Result};
use super::{BitDepth, Bitmap, BitmapHeader, BitmapInfoHeader};

pub fn from_reader<R: std::io::Read>(reader: &mut R, file_name: String) -> Result<Bitmap> {
    let mut buffer = vec![];
    reader.read_to_end(&mut buffer)?;

    if buffer.len() < HEADER_SIZE + INFO_HEADER_SIZE {
        return Err(Error::NonBitmap);
    }

    if &buffer[..2] != b"BM" {
        return Err(Error::NonBitmap);
    }

    let header = decode_header(&buffer)?;
    let info_header = decode_info_header(&buffer)?;

    // Up to here, everything was within bounds, but from now on, we could have a
    // corrupted bitmap, which then requires us to bound check everything.

    let palette = decode_palette(&info_header, &buffer)?;

    // TODO: implement the rest of formats
    let data = match info_header.bit_depth {
        BitDepth::MonoChrome => todo!(),
        BitDepth::Bit4 => decode_4_bit_colors(header.data_offset, &info_header, &palette, &buffer),
        BitDepth::Bit8 => todo!(),
        BitDepth::Bit16 => todo!(),
        BitDepth::Bit24 => todo!(),
    };

    Ok(Bitmap {
        header,
        info_header,
        palette,
        data,
        file_name,
    })
}

pub fn decode<P: AsRef<Path>>(path: P) -> Result<Bitmap> {
    let mut file = std::fs::OpenOptions::new().read(true).open(&path)?;
    from_reader(&mut file, path.as_ref().to_string_lossy().to_string())
}

fn decode_info_header(buffer: &[u8]) -> Result<BitmapInfoHeader> {
    let width: [u8; 4] = buffer[0x12..0x16].try_into().unwrap();
    let width = u32::from_le_bytes(width);

    let height: [u8; 4] = buffer[0x16..0x1A].try_into().unwrap();
    let height = u32::from_le_bytes(height);

    let bit_depth: [u8; 2] = buffer[0x1C..0x1E].try_into().unwrap();
    let bit_depth = u16::from_le_bytes(bit_depth);
    let bit_depth = BitDepth::try_from(bit_depth)?;

    let image_size: [u8; 4] = buffer[0x22..0x26].try_into().unwrap();
    let image_size = u32::from_le_bytes(image_size);

    let num_colors: [u8; 4] = buffer[0x2E..0x32].try_into().unwrap();
    let num_colors = u32::from_le_bytes(num_colors);

    let important_colors: [u8; 4] = buffer[0x32..0x36].try_into().unwrap();
    let important_colors = u32::from_le_bytes(important_colors);

    Ok(BitmapInfoHeader {
        width,
        height,
        bit_depth,
        image_size,
        num_colors,
        important_colors,
    })
}

fn decode_header(buffer: &[u8]) -> Result<BitmapHeader> {
    let file_size: [u8; 4] = buffer[0x02..0x06].try_into().unwrap();
    let file_size = u32::from_le_bytes(file_size);

    let data_offset: [u8; 4] = buffer[0x0A..0x0E].try_into().unwrap();
    let data_offset = u32::from_le_bytes(data_offset);

    Ok(BitmapHeader { file_size, data_offset })
}

fn decode_palette(info_header: &BitmapInfoHeader, buffer: &[u8]) -> Result<Vec<Color>> {
    let mut colors = vec![];

    // TODO: Handle 16Bit encoding... why microsoft?
    if info_header.bit_depth.has_palette() {
        for i in 0..info_header.num_colors as usize {
            let colors_offset = 0x36 + i * 4;
            let end = colors_offset + 3;
            let color: [u8; 3] = buffer[colors_offset..end].try_into().map_err(|_| Error::OutOfBounds)?;
            colors.push(Color::from(color));
        }
    }

    Ok(colors)
}

fn decode_4_bit_colors(
    data_offset: u32,
    info_header: &BitmapInfoHeader,
    palette: &[Color],
    buffer: &[u8],
) -> Vec<Color> {
    let height = info_header.height;
    let width = info_header.width;
    let mut colors = vec![Color::new(0, 0, 0); (width * height) as usize];

    let stride = ((width + 1) / 2 + 3) & !3;

    for row in 0..height {
        let src_row = height - 1 - row;
        let row_start = data_offset + src_row * stride;

        for col in 0..width {
            let byte_idx = row_start + (col / 2);
            let byte = buffer[byte_idx as usize];

            let color_idx = if col % 2 == 0 { byte >> 4 } else { byte & 0b1111 };

            colors[(row * width + col) as usize] = palette[color_idx as usize];
        }
    }

    colors
}
