use raylib::ffi::KeyboardKey;

use super::{Input, KeyStatus};
use crate::renderer::raylib::{HANDLE, NO_DRAWING_HANDLE};

#[derive(Default)]
pub struct RaylibInput;

impl Input for RaylibInput {
    fn poll(&self) -> KeyStatus {
        let mut key_status = KeyStatus(0);
        let handle = HANDLE.get().expect(NO_DRAWING_HANDLE).write().expect(NO_DRAWING_HANDLE);

        if handle.is_key_down(KeyboardKey::KEY_A) | handle.is_key_down(KeyboardKey::KEY_LEFT) {
            self.key_left_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_S) | handle.is_key_down(KeyboardKey::KEY_DOWN) {
            self.key_down_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_W) | handle.is_key_down(KeyboardKey::KEY_UP) {
            self.key_up_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_D) | handle.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.key_right_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_D) | handle.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.key_right_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_SPACE) {
            self.key_main_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_C) {
            self.key_secondary_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_ESCAPE) {
            self.key_pause_pressed(&mut key_status);
        }

        if handle.is_key_down(KeyboardKey::KEY_TAB) {
            self.key_select_pressed(&mut key_status);
        }

        key_status
    }
}
