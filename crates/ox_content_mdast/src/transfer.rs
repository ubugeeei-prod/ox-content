use std::{error::Error, fmt};

pub const TRANSFER_MAGIC: u32 = u32::from_le_bytes(*b"OXTR");
pub const TRANSFER_VERSION: u16 = 1;
pub const TRANSFER_HEADER_LEN: usize = 24;
pub const TRANSFER_SECTION_RECORD_LEN: usize = 12;

pub type Result<T> = std::result::Result<T, TransferError>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransferError {
    message: String,
}

impl TransferError {
    pub fn new(message: impl Into<String>) -> Self {
        Self { message: message.into() }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for TransferError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransferPayloadKind {
    Mdast = 1,
    MarkdownItTokens = 2,
    PreparedSource = 3,
}

impl TransferPayloadKind {
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    pub fn from_str_opt(value: &str) -> Option<Self> {
        match value {
            "mdast" => Some(Self::Mdast),
            "markdown-it-tokens" | "markdown_it_tokens" | "markdown-it" => {
                Some(Self::MarkdownItTokens)
            }
            "prepared-source" | "prepared_source" | "source" => Some(Self::PreparedSource),
            _ => None,
        }
    }
}

struct TransferSection {
    id: u32,
    bytes: Vec<u8>,
}

pub struct TransferBufferBuilder {
    kind: TransferPayloadKind,
    payload_version: u32,
    root_handle: u32,
    sections: Vec<TransferSection>,
}

impl TransferBufferBuilder {
    pub fn new(kind: TransferPayloadKind, payload_version: u32, root_handle: u32) -> Self {
        Self { kind, payload_version, root_handle, sections: Vec::new() }
    }

    pub fn push_section(&mut self, id: u32, bytes: Vec<u8>) {
        self.sections.push(TransferSection { id, bytes });
    }

    pub fn finish(self) -> Result<Vec<u8>> {
        let section_table_len =
            checked_mul(self.sections.len(), TRANSFER_SECTION_RECORD_LEN, "section table length")?;
        let body_offset = checked_add(TRANSFER_HEADER_LEN, section_table_len, "body offset")?;
        let body_len = self.sections.iter().try_fold(0usize, |total, section| {
            checked_add(total, section.bytes.len(), "body length")
        })?;
        let total_len = checked_add(body_offset, body_len, "total length")?;
        let mut buffer = Vec::with_capacity(total_len);

        push_u32(&mut buffer, TRANSFER_MAGIC);
        push_u16(&mut buffer, TRANSFER_VERSION);
        push_u16(&mut buffer, self.kind.as_u16());
        push_u32(&mut buffer, self.payload_version);
        push_u32(&mut buffer, as_u32(self.sections.len())?);
        push_u32(&mut buffer, self.root_handle);
        push_u32(&mut buffer, 0);

        let mut next_section_offset = body_offset;
        for section in &self.sections {
            push_u32(&mut buffer, section.id);
            push_u32(&mut buffer, as_u32(next_section_offset)?);
            push_u32(&mut buffer, as_u32(section.bytes.len())?);
            next_section_offset =
                checked_add(next_section_offset, section.bytes.len(), "section offset")?;
        }

        for section in self.sections {
            buffer.extend_from_slice(&section.bytes);
        }

        debug_assert_eq!(buffer.len(), total_len);

        Ok(buffer)
    }
}

pub fn as_u32(value: usize) -> Result<u32> {
    u32::try_from(value).map_err(|_| TransferError::new("transfer buffer overflow"))
}

fn checked_add(left: usize, right: usize, context: &str) -> Result<usize> {
    left.checked_add(right).ok_or_else(|| transfer_size_error(context))
}

fn checked_mul(left: usize, right: usize, context: &str) -> Result<usize> {
    left.checked_mul(right).ok_or_else(|| transfer_size_error(context))
}

fn transfer_size_error(context: &str) -> TransferError {
    let mut message = String::from("transfer buffer is too large while calculating ");
    message.push_str(context);
    TransferError::new(message)
}

fn push_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_u16(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("u16 slice"))
    }

    fn read_u32(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("u32 slice"))
    }

    #[test]
    fn writes_transfer_header_and_sections() {
        let mut builder = TransferBufferBuilder::new(TransferPayloadKind::Mdast, 7, 42);
        builder.push_section(1, vec![1, 2, 3]);
        builder.push_section(2, vec![4, 5]);

        let buffer = builder.finish().expect("transfer buffer should build");
        let bytes = buffer.as_ref();

        assert_eq!(read_u32(bytes, 0), TRANSFER_MAGIC);
        assert_eq!(read_u16(bytes, 4), TRANSFER_VERSION);
        assert_eq!(read_u16(bytes, 6), TransferPayloadKind::Mdast.as_u16());
        assert_eq!(read_u32(bytes, 8), 7);
        assert_eq!(read_u32(bytes, 12), 2);
        assert_eq!(read_u32(bytes, 16), 42);

        assert_eq!(read_u32(bytes, 24), 1);
        assert_eq!(read_u32(bytes, 28), 48);
        assert_eq!(read_u32(bytes, 32), 3);

        assert_eq!(read_u32(bytes, 36), 2);
        assert_eq!(read_u32(bytes, 40), 51);
        assert_eq!(read_u32(bytes, 44), 2);

        assert_eq!(&bytes[48..53], &[1, 2, 3, 4, 5]);
    }
}
