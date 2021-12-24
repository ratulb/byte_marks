//! ## ByteMarks
//!
//!

use crate::Byte;
use std::collections::HashMap;

/// An enum to represent demarcating byte pattern. Start `next` is of interest to us. End
/// signals a successful match of a byte delimiter pattern. On matching a delimiter pattern,
/// all the bytes from end of the last delimiter match(if any) till the current delimiter
/// (excluding the delimiter bytes) - would be considered as a matched bites.

pub enum ByteMarks<'a> {
    Initializer(&'a [Byte], &'a [Byte]),
    TailIndices(Option<HashMap<&'a Byte, usize>>),
    Marking(Byte),
    MarkingEnd,
    TailEnd,
    MarkingIndices(Option<HashMap<&'a Byte, usize>>),
}

impl std::fmt::Debug for ByteMarks<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Initializer(mark, tail) => write!(
                f,
                "Initializer[{:?}, {:?}]",
                String::from_utf8(mark.to_vec()),
                String::from_utf8(tail.to_vec())
            ),
            TailIndices(_) => write!(f, "TailIndices"),
            Marking(byte) => write!(f, "{}", *byte as char),
            MarkingEnd => write!(f, "MarkingEnd"),
            TailEnd => write!(f, "TailEnd"),
            MarkingIndices(_) => write!(f, "MarkingIndices"),
        }
    }
}

impl<'a> From<Byte> for ByteMarks<'a> {
    fn from(byte: Byte) -> ByteMarks<'a> {
        Marking(byte)
    }
}

use crate::ByteMarks::*;

impl<'a> ByteMarks<'a> {
    pub(crate) fn initialize(mark: &'a str, tail: &'a str) -> Self {
        let unique = Self::all_marking_unique(mark);
        let msg = unique.map(|(i, j, c)| {
            format!(
                "Mark contains duplicate character {} at indices {} and {}",
                c, i, j
            )
        });
        assert!(unique == None, "{}", msg.unwrap());

        let unique = Self::all_marking_unique(tail);
        let msg = unique.map(|(i, j, c)| {
            format!(
                "Tail contains duplicate character {} at indices {} and {}",
                c, i, j
            )
        });
        assert!(unique == None, "{}", msg.unwrap());

        ByteMarks::Initializer(mark.as_bytes(), tail.as_bytes())
    }

    pub fn all_marking_unique(s: &str) -> Option<(usize, usize, char)> {
        s.chars().enumerate().find_map(|(i, c)| {
            s.chars()
                .enumerate()
                .skip(i + 1)
                .find(|(_, other)| c == *other)
                .map(|(j, _)| (i, j, c))
        })
    }
    pub(crate) fn init_marking_indices(&self) -> Self {
        match self {
            ByteMarks::Initializer(marking_bytes, _) => {
                let mut marking_byte_indices = HashMap::new();
                for i in 0..marking_bytes.len() {
                    marking_byte_indices.insert(&marking_bytes[i], i);
                }
                ByteMarks::MarkingIndices(Some(marking_byte_indices))
            }
            _ => panic!("Forbidden"),
        }
    }

    pub(crate) fn init_tail_indices(&self) -> Self {
        match self {
            ByteMarks::Initializer(_, tail_bytes) => {
                let mut tail_byte_indices = HashMap::new();
                for i in 0..tail_bytes.len() {
                    tail_byte_indices.insert(&tail_bytes[i], i);
                }
                ByteMarks::TailIndices(Some(tail_byte_indices))
            }
            _ => panic!("Forbidden"),
        }
    }
    fn marking_byte_index(&self, byte: Byte) -> Option<&usize> {
        match self {
            ByteMarks::MarkingIndices(indices) => match indices {
                Some(map) => map.get(&byte),
                None => panic!("Marking indices not yet initialized"),
            },
            _ => panic!("Forbidden"),
        }
    }

    fn tail_byte_index(&self, byte: Byte) -> Option<&usize> {
        match self {
            ByteMarks::TailIndices(indices) => match indices {
                Some(map) => map.get(&byte),
                None => panic!("Tail indices not yet initialized"),
            },
            _ => panic!("Forbidden"),
        }
    }

    fn marking_end_byte(&self) -> Byte {
        self.marking_bytes()[self.marking_bytes_len() - 1]
    }

    pub(crate) fn marking_start_byte(&self) -> Byte {
        self.marking_bytes()[0]
    }

    pub(crate) fn marking_bytes(&self) -> &[Byte] {
        match self {
            ByteMarks::Initializer(mark, _) => mark,
            _ => panic!("Forbidden"),
        }
    }

    pub(crate) fn marking_bytes_len(&self) -> usize {
        match self {
            ByteMarks::Initializer(mark, _) => mark.len(),
            _ => panic!("Forbidden"),
        }
    }

    fn tail_end_byte(&self) -> Byte {
        self.tail_bytes()[self.tail_bytes_len() - 1]
    }

    pub(crate) fn tail_start_byte(&self) -> Byte {
        self.tail_bytes()[0]
    }

    pub(crate) fn tail_bytes(&self) -> &[Byte] {
        match self {
            ByteMarks::Initializer(_, tail) => tail,
            _ => panic!("Forbidden"),
        }
    }

    pub(crate) fn tail_bytes_len(&self) -> usize {
        match self {
            ByteMarks::Initializer(_, tail) => tail.len(),
            _ => panic!("Forbidden"),
        }
    }

    fn as_byte(&self) -> Byte {
        match self {
            Marking(v) => *v,
            _ => panic!("Only Marking will have byte value!"),
        }
    }

    fn next_marking(
        &'a self,
        initializer: &'a ByteMarks<'a>,
        indices: &'a ByteMarks<'a>,
    ) -> Option<Self> {
        match self {
            ByteMarks::Initializer(_, _) => Some(initializer.marking_start_byte().into()),
            Marking(byte) if *byte == initializer.marking_end_byte() => Some(MarkingEnd),
            Marking(byte) => indices.marking_byte_index(*byte).and_then(|index| {
                if *index + 1 < initializer.marking_bytes_len() {
                    Some(initializer.marking_bytes()[*index + 1].into())
                } else {
                    None
                }
            }),
            MarkingEnd => None,
            _ => panic!("Forbidden"),
        }
    }

    pub(crate) fn marking_matches(
        &'a self,
        initializer: &'a ByteMarks<'a>,
        indices: &'a ByteMarks<'a>,
        index: usize,
        bytes: &[u8],
    ) -> bool {
        match self {
            ByteMarks::Initializer(_, _) => {
                self.next_marking(initializer, indices)
                    .map_or(false, |next| {
                        index < bytes.len()
                            && next.as_byte() == bytes[index]
                            && next
                                .next_marking(initializer, indices)
                                .map_or(false, |next_next| {
                                    next_next.marking_matches(
                                        initializer,
                                        indices,
                                        index + 1,
                                        bytes,
                                    )
                                })
                    })
            }
            Marking(byte) => {
                index < bytes.len()
                    && bytes[index] == *byte
                    && self
                        .next_marking(initializer, indices)
                        .map_or(false, |next| {
                            next.marking_matches(initializer, indices, index + 1, bytes)
                        })
            }
            MarkingEnd => true,
            _ => false, //TODO ponder
        }
    }

    fn next_tail_marking(
        &'a self,
        initializer: &'a ByteMarks<'a>,
        indices: &'a ByteMarks<'a>,
    ) -> Option<Self> {
        match self {
            ByteMarks::Initializer(_, _) => Some(initializer.tail_start_byte().into()),
            Marking(byte) if *byte == initializer.tail_end_byte() => Some(TailEnd),
            Marking(byte) => indices.tail_byte_index(*byte).and_then(|index| {
                if *index + 1 < initializer.tail_bytes_len() {
                    Some(initializer.tail_bytes()[*index + 1].into())
                } else {
                    None
                }
            }),
            TailEnd => None,
            _ => None, //TODO ponder
        }
    }

    pub(crate) fn tail_marking_matches(
        &'a self,
        initializer: &'a ByteMarks<'a>,
        indices: &'a ByteMarks<'a>,
        index: usize,
        bytes: &[u8],
    ) -> bool {
        match self {
            ByteMarks::Initializer(_, _) => {
                self.next_tail_marking(initializer, indices)
                    .map_or(false, |next| {
                        index < bytes.len()
                            && next.as_byte() == bytes[index]
                            && next.next_tail_marking(initializer, indices).map_or(
                                false,
                                |next_next| {
                                    next_next.tail_marking_matches(
                                        initializer,
                                        indices,
                                        index + 1,
                                        bytes,
                                    )
                                },
                            )
                    })
            }
            Marking(byte) => {
                index < bytes.len()
                    && bytes[index] == *byte
                    && self
                        .next_tail_marking(initializer, indices)
                        .map_or(false, |next| {
                            next.tail_marking_matches(initializer, indices, index + 1, bytes)
                        })
            }
            TailEnd => true,
            _ => false,
        }
    }
}
