use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

use aya_cpu::memory::Addressable;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDrawHandle};
use raylib::ffi::{PixelFormat, Rectangle, Vector2};
use raylib::texture::{Image, Texture2D};
use raylib::{RaylibHandle, RaylibThread};

use super::error::Result;
use super::Renderer;
use crate::memory::{BG_MEMORY, BG_MEM_LOC, INTERFACE_MEMORY, SPRITE_MEM_LOC, TILE_MEM_LOC, UI_MEM_LOC};
use crate::PALETTE;

const TILES_WIDTH: u16 = 30;
const TILES_HEIGHT: u16 = 14;
const BYTES_PER_TILE: u16 = 32;
const SPRITE_WIDTH: u16 = 8;
const SPRITE_HEIGHT: u16 = 8;

pub static HANDLE: OnceLock<Arc<RwLock<RaylibHandle>>> = OnceLock::new();
pub static NO_DRAWING_HANDLE: &str = "tried to draw with no drawing handle";

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TextureFlags {
    Normal,
    MirrorX,
    MirrorY,
}

const X_MIRROR_MASK: u8 = 0b00000001;
const Y_MIRROR_MASK: u8 = 0b00000010;

impl IntoFlags for TextureFlags {
    fn into_flags(self) -> Vec<TextureFlags> {
        match self {
            TextureFlags::Normal => vec![TextureFlags::Normal],
            TextureFlags::MirrorX => vec![TextureFlags::MirrorX],
            TextureFlags::MirrorY => vec![TextureFlags::MirrorY],
        }
    }
}

trait IntoFlags {
    fn into_flags(self) -> Vec<TextureFlags>;
}

impl IntoFlags for u8 {
    fn into_flags(self) -> Vec<TextureFlags> {
        if self == 0 {
            return vec![TextureFlags::Normal];
        };

        let mut masks = vec![];

        if (self & X_MIRROR_MASK) == X_MIRROR_MASK {
            masks.push(TextureFlags::MirrorX);
        }

        if (self & Y_MIRROR_MASK) == Y_MIRROR_MASK {
            masks.push(TextureFlags::MirrorY);
        }

        masks
    }
}

impl From<TextureFlags> for u8 {
    fn from(value: TextureFlags) -> Self {
        value as u8
    }
}

impl std::ops::BitOr for TextureFlags {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        self as u8 | rhs as u8
    }
}

#[derive(Debug)]
pub struct RaylibRenderer {
    scale: u16,
    thread: RaylibThread,
    frame_start: Instant,
    frame_duration: Duration,
    textures: HashMap<u8, Texture2D>,
    has_cached_tiles: bool,
}

trait FromColor {
    fn to_color_array(&self) -> [u8; 4];
}

impl FromColor for (u8, u8, u8, u8) {
    fn to_color_array(&self) -> [u8; 4] {
        let (r, g, b, a) = *self;
        [r, g, b, a]
    }
}

impl RaylibRenderer {
    pub fn tile_to_texture(
        &mut self,
        handle: &mut RaylibHandle,
        tile_idx: u8,
        memory: &mut impl Addressable,
    ) -> Result<()> {
        let tile_address = TILE_MEM_LOC.0 + tile_idx as u16 * 32;

        let mut pixel_data = vec![0u8; (SPRITE_WIDTH * SPRITE_HEIGHT * 4) as usize];

        for byte_idx in 0..BYTES_PER_TILE {
            let tile_byte = memory.read(tile_address + byte_idx)?;
            let color_left = PALETTE[(tile_byte >> 4) as usize];
            let color_right = PALETTE[(tile_byte & 0xf) as usize];

            let x = (byte_idx % 4) * 2;
            let y = byte_idx / 4;

            let idx_left = ((y * SPRITE_WIDTH + x) * 4) as usize;
            pixel_data[idx_left..idx_left + 4].copy_from_slice(&color_left.to_color_array());

            let idx_right = ((y * SPRITE_WIDTH + x + 1) * 4) as usize;
            pixel_data[idx_right..idx_right + 4].copy_from_slice(&color_right.to_color_array());
        }

        let mut image = Image::gen_image_color(SPRITE_WIDTH as i32, SPRITE_HEIGHT as i32, Color::BLANK);
        image.format = PixelFormat::PIXELFORMAT_UNCOMPRESSED_R8G8B8A8 as i32;
        unsafe {
            let data_ptr = image.data as *mut u8;
            let num_bytes = (SPRITE_WIDTH * SPRITE_HEIGHT * 4) as usize;
            std::ptr::copy_nonoverlapping(pixel_data.as_ptr(), data_ptr, num_bytes);
        }

        let texture = handle.load_texture_from_image(&self.thread, &image).unwrap();
        self.textures.insert(tile_idx, texture);

        Ok(())
    }

    fn render_background(
        &mut self,
        memory: &mut impl Addressable,
        draw_handle: &mut RaylibDrawHandle,
        scale: u16,
    ) -> Result<()> {
        self.draw_memory_section(memory, draw_handle, BG_MEM_LOC.0, BG_MEMORY as u16, scale)
        //Ok(())
    }

    fn render_foreground(
        &mut self,
        memory: &mut impl Addressable,
        draw_handle: &mut RaylibDrawHandle,
        scale: u16,
    ) -> Result<()> {
        self.draw_memory_section(memory, draw_handle, BG_MEM_LOC.0, BG_MEMORY as u16, scale)
    }

    fn render_sprites(
        &mut self,
        memory: &mut impl Addressable,
        draw_handle: &mut RaylibDrawHandle,
        scale: u16,
    ) -> Result<()> {
        for i in 0..40 {
            let sprite_addr = SPRITE_MEM_LOC.0 + i * 16;
            let tile_idx = memory.read(sprite_addr)?;
            let sprite_x = memory.read(sprite_addr + 1)?;
            let sprite_y = memory.read(sprite_addr + 2)?;
            let sprite_flags = memory.read(sprite_addr + 3)?;
            let texture = self.textures.get(&tile_idx).unwrap();

            self.render_texture(
                texture,
                sprite_x as u16 * scale,
                sprite_y as u16 * scale,
                draw_handle,
                scale,
                sprite_flags,
            )?;
        }

        Ok(())
    }

    fn render_interface(
        &mut self,
        memory: &mut impl Addressable,
        draw_handle: &mut RaylibDrawHandle,
        scale: u16,
    ) -> Result<()> {
        self.draw_memory_section(memory, draw_handle, UI_MEM_LOC.0, INTERFACE_MEMORY as u16, scale)
    }

    fn draw_memory_section(
        &mut self,
        memory: &mut impl Addressable,
        draw_handle: &mut RaylibDrawHandle,
        section_location: u16,
        section_size: u16,
        scale: u16,
    ) -> Result<()> {
        for idx in 0..section_size {
            let tile_idx = memory.read(section_location + idx)?;
            let tile_x = idx % TILES_WIDTH * SPRITE_WIDTH * scale;
            let tile_y = idx / TILES_WIDTH * SPRITE_WIDTH * scale;
            self.render_tile(tile_idx, tile_x, tile_y, draw_handle, scale)?;
        }
        Ok(())
    }

    fn render_texture(
        &self,
        texture: &Texture2D,
        x: u16,
        y: u16,
        draw_handle: &mut RaylibDrawHandle,
        scale: u16,
        texture_flags: impl IntoFlags,
    ) -> Result<()> {
        let texture_flags = texture_flags.into_flags();

        // Determine if we need to flip the texture
        let mut width = texture.width as f32;
        let mut height = texture.height as f32;

        if texture_flags.contains(&TextureFlags::MirrorX) {
            width = -width;
        }
        if texture_flags.contains(&TextureFlags::MirrorY) {
            height = -height;
        }

        let source = Rectangle {
            x: x as f32,
            y: y as f32,
            width,
            height,
        };
        let dest = Rectangle {
            x: x as f32,
            y: y as f32,
            width: texture.width as f32 * scale as f32,
            height: texture.height as f32 * scale as f32,
        };
        let origin = Vector2 { x: 0.0, y: 0.0 };

        draw_handle.draw_texture_pro(texture, source, dest, origin, 0.0, Color::WHITE);
        Ok(())
    }

    fn render_tile(
        &mut self,
        tile_idx: u8,
        x: u16,
        y: u16,
        draw_handle: &mut RaylibDrawHandle,
        scale: u16,
    ) -> Result<()> {
        let texture = self.textures.get(&tile_idx).unwrap();
        self.render_texture(texture, x, y, draw_handle, scale, TextureFlags::Normal)?;
        Ok(())
    }

    fn cache_tiles(&mut self, handle: &mut RaylibHandle, memory: &mut impl Addressable) -> Result<()> {
        for idx in 0..=255 {
            self.tile_to_texture(handle, idx, memory)?;
        }
        Ok(())
    }
}

impl Renderer for RaylibRenderer {
    fn start(name: &str, fps: f32, scale: u16) -> Self {
        let (handle, thread) = raylib::init()
            .size(
                TILES_WIDTH as i32 * SPRITE_WIDTH as i32 * scale as i32,
                TILES_HEIGHT as i32 * SPRITE_WIDTH as i32 * scale as i32,
            )
            .title(name)
            .resizable()
            .build();

        let frame_start = Instant::now();
        let frame_duration = Duration::from_secs_f64(1.0 / fps as f64);

        HANDLE.get_or_init(|| Arc::new(RwLock::new(handle)));

        Self {
            scale,
            thread,
            frame_start,
            frame_duration,
            has_cached_tiles: false,
            textures: HashMap::with_capacity(255),
        }
    }

    fn should_close(&self) -> bool {
        HANDLE
            .get()
            .map(|h| h.read().unwrap().window_should_close())
            .unwrap_or(false)
    }

    fn should_draw(&self) -> bool {
        self.frame_start.elapsed() >= self.frame_duration
    }

    fn draw_frame(&mut self, memory: &mut impl Addressable) -> Result<()> {
        let mut handle = HANDLE.get().expect(NO_DRAWING_HANDLE).write().expect(NO_DRAWING_HANDLE);
        if !self.has_cached_tiles {
            self.cache_tiles(&mut handle, memory)?;
            self.has_cached_tiles = true;
        }

        let mut draw_handle = handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);

        self.render_background(memory, &mut draw_handle, self.scale)?;
        self.render_sprites(memory, &mut draw_handle, self.scale)?;
        self.render_foreground(memory, &mut draw_handle, self.scale)?;
        self.render_interface(memory, &mut draw_handle, self.scale)?;

        self.frame_start = Instant::now();
        Ok(())
    }
}
