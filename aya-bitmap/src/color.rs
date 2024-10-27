/// `Color` used to describe the palette of a bitmap image
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl std::fmt::LowerHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl std::fmt::UpperHex for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl From<[u8; 3]> for Color {
    fn from([b, g, r]: [u8; 3]) -> Self {
        Self { r, g, b }
    }
}

impl From<Color> for [u8; 3] {
    fn from(color: Color) -> Self {
        let Color { r, g, b } = color;
        [r, g, b]
    }
}

impl From<Color> for (u8, u8, u8) {
    fn from(color: Color) -> Self {
        let Color { r, g, b } = color;
        (r, g, b)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Self { r, g, b }
    }
}

impl From<&(u8, u8, u8)> for Color {
    fn from((r, g, b): &(u8, u8, u8)) -> Self {
        Self { r: *r, g: *g, b: *b }
    }
}
