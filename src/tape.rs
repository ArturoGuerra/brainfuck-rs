pub struct Tape(Vec<u8>);

impl Default for Tape {
    fn default() -> Self {
        Self(vec![0u8; 30 * 1000])
    }
}

impl Tape {
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut u8> {
        self.0.get_mut(idx)
    }

    pub fn get(&mut self, idx: usize) -> Option<&u8> {
        self.0.get(idx)
    }
}
