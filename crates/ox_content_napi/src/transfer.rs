use std::ptr;

use napi::bindgen_prelude::Uint8Array;

pub const TRANSFER_MAGIC: u32 = u32::from_le_bytes(*b"OXTR");
pub const TRANSFER_VERSION: u16 = 1;
pub const TRANSFER_HEADER_LEN: usize = 24;
pub const TRANSFER_SECTION_RECORD_LEN: usize = 12;

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

    pub fn from_str(value: &str) -> Option<Self> {
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

    pub fn finish(self) -> napi::Result<Uint8Array> {
        let section_table_len = self.sections.len() * TRANSFER_SECTION_RECORD_LEN;
        let body_offset = TRANSFER_HEADER_LEN + section_table_len;
        let body_len = self.sections.iter().map(|section| section.bytes.len()).sum::<usize>();
        let total_len = body_offset + body_len;
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
            next_section_offset += section.bytes.len();
        }

        for section in self.sections {
            buffer.extend_from_slice(&section.bytes);
        }

        debug_assert_eq!(buffer.len(), total_len);

        into_external_uint8_array(buffer)
    }
}

pub fn as_u32(value: usize) -> napi::Result<u32> {
    u32::try_from(value).map_err(|_| napi::Error::from_reason("transfer buffer overflow"))
}

fn push_u16(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_le_bytes());
}

#[allow(unsafe_code)]
fn into_external_uint8_array(buffer: Vec<u8>) -> napi::Result<Uint8Array> {
    let len = buffer.len();
    let ptr = Box::into_raw(buffer.into_boxed_slice()).cast::<u8>();
    let array = unsafe {
        Uint8Array::with_external_data(ptr, len, move |ptr, len| {
            let slice = ptr::slice_from_raw_parts_mut(ptr, len);
            drop(Box::from_raw(slice));
        })
    };
    Ok(array)
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
