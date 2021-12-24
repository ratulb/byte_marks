//! ## ByteMarker
//!
use crate::Byte;
use crate::ByteMarks;

pub struct ByteMarker<'a> {
    initializer: ByteMarks<'a>,
    marks: ByteMarks<'a>,
    tail: Option<ByteMarks<'a>>,
}

impl<'a> ByteMarker<'a> {
    pub fn new(mark: &'a str, tail: &'a str) -> Self {
        assert_ne!(mark, "", "Mark should not be empty!");
        assert_ne!(
            mark, tail,
            "Mark {} and tail {} same! Should not be",
            mark, tail
        );
        let initializer = ByteMarks::initialize(mark, tail);
        let marks = initializer.init_marking_indices();
        let tail = if initializer.tail_bytes_len() > 0 {
            Some(initializer.init_tail_indices())
        } else {
            None
        };
        Self {
            initializer,
            marks,
            tail,
        }
    }

    pub fn unmark<'b>(&self, bytes: &'b [Byte]) -> Option<(Vec<&'b [Byte]>, Option<&'b [Byte]>)> {
        if bytes.is_empty() {
            return None;
        }
        let mut unmarked = Vec::with_capacity(bytes.len());
        let mut processed_bytes = 0;

        let start_byte = self.initializer.marking_start_byte();
        let mark_size = self.initializer.marking_bytes_len();

        for index in 0..bytes.len() {
            if bytes[index] == start_byte
                && self
                    .initializer
                    .marking_matches(&self.initializer, &self.marks, index, bytes)
            {
                unmarked.push(&bytes[processed_bytes..index]);
                processed_bytes = index + mark_size;
            }
            match self.tail {
                None => continue,
                Some(ref tail) => {
                    if bytes[index] == self.initializer.tail_start_byte()
                        && self.initializer.tail_marking_matches(
                            &self.initializer,
                            tail,
                            index,
                            bytes,
                        )
                    {
                        unmarked.push(&bytes[processed_bytes..index]);
                        return Some((unmarked, None));
                    }
                }
            }
        }
        let left_over = match &bytes[processed_bytes..] {
            [remained @ ..] if !remained.is_empty() => Some(remained),
            _ => None,
        };
        Some((unmarked, left_over))
    }

    pub fn mark_bytes(&self, bytes: &mut Vec<Byte>) {
        bytes.extend(self.initializer.marking_bytes());
    }

    pub fn erase_mark(&self, bytes: &mut Vec<Byte>) {
        bytes.truncate(bytes.len() - self.initializer.marking_bytes_len());
    }

    pub fn mark_tail(&self, bytes: &mut Vec<Byte>) {
        bytes.extend(self.initializer.tail_bytes());
    }

    pub fn erase_tail(&self, bytes: &mut Vec<Byte>) {
        bytes.truncate(bytes.len() - self.initializer.tail_bytes_len());
    }

    pub fn concat_byte(first: &[Byte], second: &[Byte]) -> Vec<Byte> {
        [first, second].concat()
    }
}
