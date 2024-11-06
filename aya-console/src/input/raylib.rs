use raylib::ffi::KeyboardKey;
use raylib::RaylibHandle;

use super::{Input, KeyStatus};

pub struct RaylibInput<'input> {
    handle: &'input RaylibHandle,
}

impl<'input> RaylibInput<'input> {
    pub fn new(handle: &'input RaylibHandle) -> Self {
        Self { handle }
    }
}

impl Input for RaylibInput<'_> {
    fn poll(&self) -> KeyStatus {
        let mut key_status = KeyStatus(0);

        if self.handle.is_key_down(KeyboardKey::KEY_A) | self.handle.is_key_down(KeyboardKey::KEY_LEFT) {
            self.key_left_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_S) | self.handle.is_key_down(KeyboardKey::KEY_DOWN) {
            self.key_down_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_W) | self.handle.is_key_down(KeyboardKey::KEY_UP) {
            self.key_up_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_D) | self.handle.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.key_right_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_D) | self.handle.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.key_right_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_SPACE) {
            self.key_main_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_C) {
            self.key_secondary_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_ESCAPE) {
            self.key_pause_pressed(&mut key_status);
        }

        if self.handle.is_key_down(KeyboardKey::KEY_TAB) {
            self.key_select_pressed(&mut key_status);
        }

        key_status
    }
}
