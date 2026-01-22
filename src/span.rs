#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    pub start: usize, // byte offset
    pub end: usize,   // byte offset (exclusive)
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
