use std::time::{Duration, Instant};

use aya_console::memory::{
    BG_MEMORY, BG_MEM_LOC, INTERFACE_MEMORY, SPRITE_MEMORY, SPRITE_MEM_LOC, TILE_MEM_LOC, UI_MEM_LOC,
};
use aya_console::PALETTE;
use aya_cpu::memory::Addressable;
use raylib::color::Color;
use raylib::drawing::{RaylibDraw, RaylibDrawHandle};
use raylib::{RaylibHandle, RaylibThread};

use super::error::Result;
use super::Renderer;

const TILES_WIDTH: u16 = 30;
const TILES_HEIGHT: u16 = 14;
const BYTES_PER_TILE: u16 = 32;
const SPRITE_WIDTH: u16 = 8;

#[derive(Debug)]
pub struct RaylibRenderer {
    scale: u16,
    thread: RaylibThread,
    handle: RaylibHandle,
    frame_start: Instant,
    frame_duration: Duration,
}

impl RaylibRenderer {
    pub fn handle(&self) -> &RaylibHandle {
        &self.handle
    }
}

impl Renderer for RaylibRenderer {
    fn start(name: &str, fps: f64, scale: u16) -> Self {
        let (handle, thread) = raylib::init()
            .size(
                TILES_WIDTH as i32 * SPRITE_WIDTH as i32 * scale as i32,
                TILES_HEIGHT as i32 * SPRITE_WIDTH as i32 * scale as i32,
            )
            .title(name)
            .resizable()
            .build();

        let frame_start = Instant::now();
        let frame_duration = Duration::from_secs_f64(1.0 / fps);

        Self {
            scale,
            thread,
            handle,
            frame_start,
            frame_duration,
        }
    }

    fn should_close(&self) -> bool {
        self.handle.window_should_close()
    }

    fn should_draw(&self) -> bool {
        self.frame_start.elapsed() >= self.frame_duration
    }

    fn draw_frame(&mut self, memory: &mut impl Addressable) -> Result<()> {
        let mut draw_handle = self.handle.begin_drawing(&self.thread);
        draw_handle.clear_background(Color::BLACK);

        //for y in 0..TILES_HEIGHT {
        //    for x in 0..TILES_WIDTH {
        //        let tile_x = x * SPRITE_WIDTH * self.scale;
        //        let tile_y = y * SPRITE_WIDTH * self.scale;
        //        if (x + y) % 2 == 0 {
        //            render_tile(tile_x, tile_y, 7 * BYTES_PER_TILE, memory, &mut draw_handle, self.scale)?;
        //        } else {
        //            render_tile(tile_x, tile_y, 6 * BYTES_PER_TILE, memory, &mut draw_handle, self.scale)?;
        //        }
        //    }
        //}

        render_background(memory, &mut draw_handle, self.scale)?;
        render_sprites(memory, &mut draw_handle, self.scale)?;
        render_interface(memory, &mut draw_handle, self.scale)?;
        self.frame_start = Instant::now();
        Ok(())
    }
}

fn render_background(memory: &mut impl Addressable, draw_handle: &mut RaylibDrawHandle, scale: u16) -> Result<()> {
    draw_memory_section(memory, draw_handle, BG_MEM_LOC.0, BG_MEMORY as u16, scale)
}

fn render_sprites(memory: &mut impl Addressable, draw_handle: &mut RaylibDrawHandle, scale: u16) -> Result<()> {
    for i in 0..40 {
        let sprite_addr = SPRITE_MEM_LOC.0 + i * 16;
        let tile_idx = memory.read(sprite_addr)?;
        let sprite_x = memory.read(sprite_addr + 1)?;
        let sprite_y = memory.read(sprite_addr + 2)?;

        render_tile(
            sprite_x as u16,
            sprite_y as u16,
            TILE_MEM_LOC.0 + (tile_idx as u16) * 32,
            memory,
            draw_handle,
            scale,
        )?;
    }

    Ok(())
}

fn render_interface(memory: &mut impl Addressable, draw_handle: &mut RaylibDrawHandle, scale: u16) -> Result<()> {
    draw_memory_section(memory, draw_handle, UI_MEM_LOC.0, INTERFACE_MEMORY as u16, scale)
}

fn draw_memory_section(
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
        let tile_address = TILE_MEM_LOC.0 + tile_idx as u16 * 32;
        render_tile(tile_x, tile_y, tile_address, memory, draw_handle, scale)?;
    }
    Ok(())
}

fn render_tile(
    x: u16,
    y: u16,
    tile_address: u16,
    memory: &mut impl Addressable,
    draw_handle: &mut RaylibDrawHandle,
    scale: u16,
) -> Result<()> {
    for byte_idx in 0..BYTES_PER_TILE {
        let tile_byte = memory.read(tile_address + byte_idx)?;
        let color_left = PALETTE[(tile_byte >> 4) as usize];
        let color_right = PALETTE[(tile_byte & 0xf) as usize];

        let left_x = x + ((byte_idx % 4) * 2) * scale;
        let right_x = left_x + scale;
        let y = y + byte_idx / 4 * scale;

        draw_handle.draw_rectangle(
            left_x as i32,
            y as i32,
            scale as i32,
            scale as i32,
            Color::from(color_left),
        );

        draw_handle.draw_rectangle(
            right_x as i32,
            y as i32,
            scale as i32,
            scale as i32,
            Color::from(color_right),
        );
    }

    Ok(())
}
