pub struct LineIndex {
    line_starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self { line_starts }
    }

    pub fn position(&self, offset: usize) -> (u32, u32) {
        match self.line_starts.binary_search(&offset) {
            Ok(idx) => (idx as u32 + 1, 1),
            Err(idx) => {
                let line = idx as u32; // idx is the first start after offset.
                let line_start = self.line_starts[idx - 1];
                let column = (offset - line_start) as u32 + 1;
                (line, column)
            }
        }
    }
}
