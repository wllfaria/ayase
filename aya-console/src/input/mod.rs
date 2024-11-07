mod raylib;

pub use raylib::RaylibInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct KeyStatus(u8);

impl std::fmt::Display for KeyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl KeyStatus {
    pub fn reset() -> Self {
        Self(0)
    }

    pub fn mask_on(&mut self, bit: u8) {
        self.0 |= 1 << bit;
    }
}

impl From<KeyStatus> for u8 {
    fn from(value: KeyStatus) -> Self {
        value.0
    }
}

pub trait Input {
    fn poll(&self) -> KeyStatus;

    fn key_left_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(7);
    }

    fn key_down_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(6);
    }

    fn key_up_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(5);
    }

    fn key_right_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(4);
    }

    fn key_main_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(3);
    }

    fn key_secondary_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(2);
    }

    fn key_pause_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(1);
    }

    fn key_select_pressed(&self, status: &mut KeyStatus) {
        status.mask_on(0);
    }
}
