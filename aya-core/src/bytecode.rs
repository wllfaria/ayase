pub struct Loader {}

impl Loader {
    pub fn load<S: AsRef<str>>(source: S) -> Vec<u8> {
        source
            .as_ref()
            .trim()
            .lines()
            .flat_map(|line| {
                line.split_whitespace()
                    .map(|part| u8::from_str_radix(part, 16).unwrap())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }
}
