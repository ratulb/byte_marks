#[cfg(feature = "std")]
use std::{
    fs::File,
    io::{BufRead, BufReader, Result},
};
#[cfg(feature = "default_async")]
use {
    async_std::{
        fs::File as OtherFile,
        io::{BufRead as OtherBufRead, BufReader as OtherBufReader, Result as OtherResult},
    },
    futures::io::AsyncBufReadExt,
};

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
        let mut mark_indices = HashMap::new();
        for i in 0..MARKS.len() {
            mark_indices.insert(&MARKS[i], i);
        }
        mark_indices
    };
}

pub fn mark_bytes(bytes: &mut Vec<u8>) {
    bytes.extend(*MARKS);
}

pub fn erase_mark(bytes: &mut Vec<u8>) {
    bytes.truncate(bytes.len() - MARKS.len());
}

fn buffered_bytes(f: &str) -> Result<Vec<u8>> {
    let f = File::open(f)?;
    let mut reader = BufReader::new(f);
    let buffered = reader.fill_buf()?;
    let buffered = buffered.to_vec();
    reader.consume(buffered.len());
    Ok(buffered)
}
#[cfg(feature = "default_async")]
async fn default_async_buffered_bytes(f: &str) -> OtherResult<Vec<u8>> {
    let f = OtherFile::open(f).await?;
    let mut reader = OtherBufReader::new(f);
    type_of(&reader);
    let buffered = reader.fill_buf().await?;
    let buffered = buffered.to_vec();
    reader.consume_unpin(buffered.len());
    Ok(buffered)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marks_unmark_test_1() -> Result<()> {
        let f = "msg_empty.txt";
        let buf = buffered_bytes(f)?;
        let unmarked = Marks::unmark(&buf);
        if let Some(unmarked) = unmarked {
            for i in 0..unmarked.0.len() {
                println!("The msg = {:?}", String::from_utf8(unmarked.0[i].to_vec()));
            }
        }
        Ok(())
    }
    #[cfg(feature = "default_async")]
    #[async_std::test]
    async fn other_marks_unmark_test_1() -> OtherResult<()> {
        let f = "msg_empty.txt";
        let buf = default_async_buffered_bytes(f).await?;
        let unmarked = Marks::unmark(&buf);
        if let Some(unmarked) = unmarked {
            for i in 0..unmarked.0.len() {
                println!("The msg = {:?}", String::from_utf8(unmarked.0[i].to_vec()));
            }
        }
        Ok(())
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum Marks {
    Start,
    Marking(Byte),
    End,
}

impl std::fmt::Display for Marks {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Start => write!(f, "Start"),
            End => write!(f, "End"),
            Marking(c) => write!(f, "{}", *c as char),
        }
    }
}

impl std::fmt::Debug for Marks {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Start => write!(f, "Start"),
            End => write!(f, "End"),
            Marking(c) => write!(f, "{}", *c as char),
        }
    }
}

pub(crate) type Byte = u8;
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

    #[inline(always)]
    pub fn start_mark() -> Marks {
        MARKS[0].into()
    }

    #[inline(always)]
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
    pub fn unmark(bytes: &[u8]) -> Option<(Vec<&[u8]>, usize)> {
        if bytes.is_empty() {
            return None;
        }
        let mut unmarked = Vec::new();
        let mut consumed_segments = 0;
        let start_byte = Self::start_mark().as_byte();
        let marks_size: usize = MARKS.len();
        for i in 0..bytes.len() {
            if bytes[i] == start_byte && Start.matches(i, bytes) {
                unmarked.push(&bytes[consumed_segments..i]);
                consumed_segments = i + marks_size;
            }
        }
        Some((unmarked, consumed_segments))
    }
}

impl From<Byte> for Marks {
    fn from(byte: Byte) -> Marks {
        Marking(byte)
    }
}
