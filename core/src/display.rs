pub struct Display {
    pixel_rows: [u64; 32],
}

impl Display {
    pub fn new() -> Self {
        Self { pixel_rows: [0; 32] }
    }

    pub fn toggle(&mut self, x: u8, y: u8) -> bool {
        let mask: u64 = 1 << (63 - x);
        self.pixel_rows[y as usize] ^= mask;

        self.pixel_rows[y as usize] & mask != 0
    }

    pub fn pixel_rows(&self) -> &[u64; 32] {
        &self.pixel_rows
    }

    pub fn clear(&mut self) {
        self.pixel_rows = [0; 32];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toggle() {
        let mut display = Display::new();
        display.toggle(2, 18);
        display.toggle(16, 23);

        assert_eq!(display.pixel_rows[18], 1 << 61);
        assert_eq!(display.pixel_rows[23], 1 << 47);
    }

    #[test]
    fn test_clear() {
        let mut display = Display::new();
        display.toggle(18, 9);
        display.toggle(54, 10);
        display.clear();

        assert!(display.pixel_rows.iter().all(|r|*r == 0));
    }
}
