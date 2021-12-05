use lazy_static::lazy_static;
use std::collections::HashMap;
use std::env;

//Note the characters in the byte mark pattern should not repeat
lazy_static! {
    pub static ref MARKS: &'static [u8] = Box::leak(
        env::var("byte_marks")
            .unwrap_or(include_str!("byte_marks").to_string())
            .into_boxed_str()
    )
    .as_bytes();
    pub static ref INDICES: HashMap<&'static u8, usize> = {
        let mut indices = HashMap::new();
        for i in 0..MARKS.len() {
            indices.insert(&MARKS[i], i);
        }
        indices
    };
}

pub enum Marks {
    Start,
    Marking(Byte),
    End,
}

pub(crate) type Byte = u8;
use crate::Marks::*;

impl Marks {
    #[inline(always)]
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
    #[inline(always)]
    pub fn as_byte(&self) -> Byte {
        match self {
            Marking(v) => *v,
            _ => panic!("Only Marking will have byte value!"),
        }
    }

    #[inline(always)]
    pub fn start_mark() -> Marks {
        MARKS[0].into()
    }

    #[inline(always)]
    pub fn last_byte() -> Byte {
        MARKS[MARKS.len() - 1]
    }

    #[inline(always)]
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

    pub fn unmark(bytes: &[u8]) -> Option<(Vec<&[u8]>, usize)> {
        if bytes.is_empty() {
            return None;
        }
        let mut unmarked = Vec::new();
        let mut processed_segments = 0;
        let start_byte = Self::start_mark().as_byte();
        let marks_size: usize = MARKS.len();
        for i in 0..bytes.len() {
            if bytes[i] == start_byte && Start.matches(i, bytes) {
                unmarked.push(&bytes[processed_segments..i]);
                processed_segments = i + marks_size;
            }
        }
        Some((unmarked, processed_segments))
    }

    #[inline(always)]
    pub fn mark_bytes(bytes: &mut Vec<u8>) {
        bytes.extend(*MARKS);
    }

    #[inline(always)]
    pub fn erase_mark(bytes: &mut Vec<u8>) {
        bytes.truncate(bytes.len() - MARKS.len());
    }
}

impl From<Byte> for Marks {
    #[inline(always)]
    fn from(byte: Byte) -> Marks {
        Marking(byte)
    }
}
