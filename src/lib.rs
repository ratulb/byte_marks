//! ## byte_marks
//!
//! `byte_marks` is a configurable, light weight and intuitive bytes' boundary marker for
//! transmitting and receiving bytes from network/files. The demarcating byte pattern is
//! configured via a file called `byte_marks` or a an environment variable named similarly.
//! The characters in the byte mark pattern should not repeat. While trying to demarcate bytes
//! its possible that - no progress is being made - its an indication that there may be no
//! byte pattern in the stream being read or default buffer length has been hit without
//! encountering any pattern delimiter.
//!

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;

lazy_static! {
    pub static ref MARKS: &'static [u8] = Box::leak({
        let mut markings = env::var("byte_marks").unwrap_or(include_str!("byte_marks").to_string());
        trim_newline(&mut markings);
        markings.into_boxed_str()
    })
    .as_bytes();
    pub static ref INDICES: HashMap<&'static u8, usize> = {
        let mut indices = HashMap::new();
        for i in 0..MARKS.len() {
            indices.insert(&MARKS[i], i);
        }
        indices
    };
}

pub fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

pub(crate) type Byte = u8;
/// An enum to represent demarcating byte pattern. Start `next` is of interest to us. End
/// signals a successful match of a byte delimiter pattern. On matching a delimiter pattern,
/// all the bytes from end of the last delimiter match(if any) till the current delimiter
/// (excluding the delimiter bytes) - would be considered as a matched bites.

pub enum Marks {
    Start,
    Marking(Byte),
    End,
}

use crate::Marks::*;
impl Marks {
    pub fn next(&self) -> Option<Self> {
        match self {
            Start => Some(Self::start_mark()),
            Marking(byte) if *byte == Self::last_byte() => Some(End),
            Marking(byte) => INDICES.get(byte).and_then(|index| {
                if *index + 1 < MARKS.len() {
                    Some(MARKS[*index + 1].into())
                } else {
                    None
                }
            }),
            End => None,
        }
    }

    pub fn as_byte(&self) -> Byte {
        match self {
            Marking(v) => *v,
            _ => panic!("Only Marking will have byte value!"),
        }
    }

    pub fn start_mark() -> Marks {
        MARKS[0].into()
    }

    pub fn last_byte() -> Byte {
        MARKS[MARKS.len() - 1]
    }

    pub fn matches(&self, index: usize, bytes: &[u8]) -> bool {
        match self {
            Start => self.next().map_or(false, |next| {
                index < bytes.len()
                    && next.as_byte() == bytes[index]
                    && next
                        .next()
                        .map_or(false, |next_next| next_next.matches(index + 1, bytes))
            }),
            Marking(byte) => {
                index < bytes.len()
                    && bytes[index] == *byte
                    && self
                        .next()
                        .map_or(false, |next| next.matches(index + 1, bytes))
            }
            End => true,
        }
    }

    pub fn unmark(bytes: &[u8]) -> Option<(Vec<&[u8]>, Option<&[u8]>)> {
        if bytes.is_empty() {
            return None;
        }

        let mut unmarked = Vec::with_capacity(bytes.len());
        let mut processed_bytes = 0;
        let start_byte = Self::start_mark().as_byte();
        let marks_size = MARKS.len();

        for i in 0..bytes.len() {
            if bytes[i] == start_byte && Start.matches(i, bytes) {
                unmarked.push(&bytes[processed_bytes..i]);
                processed_bytes = i + marks_size;
            }
        }
        let left_over = match &bytes[processed_bytes..] {
            [remained @ ..] if remained.len() > 0 => Some(remained),
            _ => None,
        };
        Some((unmarked, left_over))
    }

    pub fn mark_bytes(bytes: &mut Vec<u8>) {
        bytes.extend(*MARKS);
    }

    pub fn erase_mark(bytes: &mut Vec<u8>) {
        bytes.truncate(bytes.len() - MARKS.len());
    }
    pub fn concat_u8(first: &[u8], second: &[u8]) -> Vec<u8> {
        [first, second].concat()
    }
}

impl From<Byte> for Marks {
    fn from(byte: Byte) -> Marks {
        Marking(byte)
    }
}
