use super::SIZE_CLASS_BUCKETS;

/// Histogram of allocations bucketed by power-of-two size class.
#[derive(Debug, Clone)]
pub struct SizeHistogram {
    pub buckets: [u64; SIZE_CLASS_BUCKETS],
}

impl SizeHistogram {
    /// Returns the human-readable label for bucket `i`, e.g. `"32..64"`.
    #[must_use]
    pub fn bucket_label(i: usize) -> String {
        if i == 0 {
            return "0".into();
        }
        if i == 1 {
            return "1".into();
        }
        let lo = 1u64 << (i - 1).min(63);
        let hi = 1u64 << i.min(63);
        let mut label = String::with_capacity(42);
        push_u64(&mut label, lo);
        label.push_str("..");
        push_u64(&mut label, hi);
        label
    }

    /// Iterates `(label, count)` pairs, skipping empty buckets.
    pub fn iter_nonempty(&self) -> impl Iterator<Item = (String, u64)> + '_ {
        self.buckets
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(i, c)| (Self::bucket_label(i), *c))
    }
}

fn push_u64(output: &mut String, mut value: u64) {
    // Stack buffer is filled only with ASCII `'0'..='9'`. Interpret that
    // suffix as UTF-8, and copy bytes as chars if the check ever fails
    // rather than aborting a profiler report.
    let mut buffer = [0_u8; 20];
    let mut cursor = buffer.len();
    loop {
        cursor -= 1;
        buffer[cursor] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    match std::str::from_utf8(&buffer[cursor..]) {
        Ok(digits) => output.push_str(digits),
        Err(_) => output.extend(buffer[cursor..].iter().copied().map(char::from)),
    }
}

#[inline]
pub(super) fn size_class_bucket(size: u64) -> usize {
    // Bucket by floor(log2(size)). Bucket 0 catches size 0 (rare but legal),
    // bucket 1 catches sizes 1..=1, bucket k catches sizes [2^(k-1), 2^k).
    if size == 0 {
        return 0;
    }
    let log2 = (u64::BITS - 1 - size.leading_zeros()) as usize;
    log2.min(SIZE_CLASS_BUCKETS - 1) + usize::from(size > 1)
}
