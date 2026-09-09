//! Prepared exact-match candidates, preserving longest-match precedence.
#[derive(Clone)]
pub(super) struct Terms {
    entries: Vec<(String, String)>,
    starts: Option<Box<[usize; 257]>>,
}

impl Terms {
    pub(super) fn new(mut entries: Vec<(String, String)>) -> Self {
        entries.sort_by(|a, b| {
            a.0.as_bytes()[0]
                .cmp(&b.0.as_bytes()[0])
                .then_with(|| b.0.len().cmp(&a.0.len()))
                .then_with(|| a.0.cmp(&b.0))
        });
        if entries.len() <= 8 {
            return Self { entries, starts: None };
        }
        let mut starts = Box::new([0; 257]);
        for (term, _) in &entries {
            starts[usize::from(term.as_bytes()[0]) + 1] += 1;
        }
        for i in 1..starts.len() {
            starts[i] += starts[i - 1];
        }
        Self { entries, starts: Some(starts) }
    }

    pub(super) fn entries(&self) -> &[(String, String)] {
        &self.entries
    }
    pub(super) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub(super) fn starting_with(&self, byte: u8) -> &[(String, String)] {
        let Some(starts) = &self.starts else {
            return &self.entries;
        };
        let bucket = usize::from(byte);
        &self.entries[starts[bucket]..starts[bucket + 1]]
    }
}
