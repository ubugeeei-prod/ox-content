pub struct StringBuilder {
    output: String,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self { output: String::new() }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { output: String::with_capacity(capacity) }
    }

    pub fn push_str(&mut self, value: &str) {
        self.output.push_str(value);
    }

    pub fn push_char(&mut self, value: char) {
        self.output.push(value);
    }

    pub fn push_usize(&mut self, value: usize) {
        let mut buffer = [0_u8; 20];
        let mut cursor = buffer.len();
        let mut rest = value;

        loop {
            cursor -= 1;
            buffer[cursor] = b'0' + (rest % 10) as u8;
            rest /= 10;
            if rest == 0 {
                break;
            }
        }

        let digits = std::str::from_utf8(&buffer[cursor..]).expect("digits are valid utf-8");
        self.output.push_str(digits);
    }

    #[cfg(test)]
    pub fn push_u128(&mut self, value: u128) {
        let mut buffer = [0_u8; 39];
        let mut cursor = buffer.len();
        let mut rest = value;

        loop {
            cursor -= 1;
            buffer[cursor] = b'0' + (rest % 10) as u8;
            rest /= 10;
            if rest == 0 {
                break;
            }
        }

        let digits = std::str::from_utf8(&buffer[cursor..]).expect("digits are valid utf-8");
        self.output.push_str(digits);
    }

    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    pub fn into_string(self) -> String {
        self.output
    }
}

pub fn join2(first: &str, second: &str) -> String {
    let mut out = StringBuilder::with_capacity(first.len() + second.len());
    out.push_str(first);
    out.push_str(second);
    out.into_string()
}

pub fn join3(first: &str, second: &str, third: &str) -> String {
    let mut out = StringBuilder::with_capacity(first.len() + second.len() + third.len());
    out.push_str(first);
    out.push_str(second);
    out.push_str(third);
    out.into_string()
}

pub fn join4(first: &str, second: &str, third: &str, fourth: &str) -> String {
    let mut out =
        StringBuilder::with_capacity(first.len() + second.len() + third.len() + fourth.len());
    out.push_str(first);
    out.push_str(second);
    out.push_str(third);
    out.push_str(fourth);
    out.into_string()
}

pub fn join5(first: &str, second: &str, third: &str, fourth: &str, fifth: &str) -> String {
    let mut out = StringBuilder::with_capacity(
        first.len() + second.len() + third.len() + fourth.len() + fifth.len(),
    );
    out.push_str(first);
    out.push_str(second);
    out.push_str(third);
    out.push_str(fourth);
    out.push_str(fifth);
    out.into_string()
}
