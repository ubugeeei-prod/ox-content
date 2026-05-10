// @author kazuya kawaguchi (a.k.a. kazupon)
// @license MIT
//

//! Errors raised by the lazy decoder.

use core::fmt;

use crate::format::kind::UnknownKind;

/// Errors that can occur while decoding a Binary AST byte stream.
///
/// The lazy decoder validates only the Header eagerly; per-node and
/// per-string failures surface here as decoding proceeds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// The input slice is shorter than the 40-byte Header.
    TooShort {
        /// Actual byte length of the input slice.
        actual: usize,
        /// Minimum length required (always 40 for this variant).
        required: usize,
    },
    /// The buffer's major version disagrees with the decoder's supported
    /// major version. Different majors are intentionally incompatible — see
    /// `format.md` "Header" version-rules table.
    IncompatibleMajor {
        /// Major version stored in the buffer's Header byte 0.
        buffer_major: u8,
        /// Major version this decoder was compiled against.
        decoder_major: u8,
    },
    /// The Node Data type tag is the reserved `0b11` value. Phase 1 decoders
    /// must reject this; later phases may assign meaning to it.
    UnsupportedTypeTag {
        /// Index of the offending node record.
        node_index: u32,
        /// Raw 2-bit tag value (always `0b11` today).
        tag: u32,
    },
    /// The Kind byte does not correspond to any defined node kind.
    UnknownKind {
        /// Index of the offending node record.
        node_index: u32,
        /// Raw Kind byte that failed to decode.
        kind_byte: u8,
    },
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::TooShort { actual, required } => {
                write!(f, "input is too short: got {actual} bytes, need at least {required}")
            }
            DecodeError::IncompatibleMajor { buffer_major, decoder_major } => write!(
                f,
                "incompatible major version: buffer is v{buffer_major}, decoder supports v{decoder_major}"
            ),
            DecodeError::UnsupportedTypeTag { node_index, tag } => {
                write!(f, "node[{node_index}] uses reserved Node Data type tag 0b{tag:02b}")
            }
            DecodeError::UnknownKind { node_index, kind_byte } => {
                write!(f, "node[{node_index}] has unknown Kind 0x{kind_byte:02X}")
            }
        }
    }
}

impl From<UnknownKind> for DecodeError {
    fn from(err: UnknownKind) -> Self {
        DecodeError::UnknownKind { node_index: 0, kind_byte: err.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_too_short() {
        let err = DecodeError::TooShort { actual: 8, required: 40 };
        assert_eq!(err.to_string(), "input is too short: got 8 bytes, need at least 40");
    }

    #[test]
    fn display_incompatible_major() {
        let err = DecodeError::IncompatibleMajor { buffer_major: 2, decoder_major: 1 };
        assert_eq!(
            err.to_string(),
            "incompatible major version: buffer is v2, decoder supports v1"
        );
    }

    #[test]
    fn display_unsupported_type_tag() {
        let err = DecodeError::UnsupportedTypeTag { node_index: 17, tag: 0b11 };
        assert_eq!(err.to_string(), "node[17] uses reserved Node Data type tag 0b11");
    }

    #[test]
    fn display_unknown_kind() {
        let err = DecodeError::UnknownKind { node_index: 5, kind_byte: 0x40 };
        assert_eq!(err.to_string(), "node[5] has unknown Kind 0x40");
    }

    #[test]
    fn from_unknown_kind_preserves_byte() {
        let conv: DecodeError = UnknownKind(0x42).into();
        match conv {
            DecodeError::UnknownKind { kind_byte, .. } => assert_eq!(kind_byte, 0x42),
            _ => panic!("expected UnknownKind variant"),
        }
    }
}
