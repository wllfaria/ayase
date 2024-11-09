fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rom_file = std::env::args().nth(1).unwrap();
    aya_console::run(rom_file)
}
