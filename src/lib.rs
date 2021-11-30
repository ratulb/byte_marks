#[cfg(feature = "std")]
use std::{
    fs::File,
    io::{BufRead, BufReader, Result},
};

const MARK: &[u8] = "sUfFiX".as_bytes();
const MARK_SIZE: usize = MARK.len();

pub fn mark_bytes(bytes: &mut Vec<u8>) {
    bytes.extend(MARK);
}

pub fn erase_mark(bytes: &mut Vec<u8>) {
    bytes.truncate(bytes.len() - MARK_SIZE);
}

fn buffered_bytes(f: &str) -> Result<Vec<u8>> {
    let f = File::open(f)?;
    let mut reader = BufReader::new(f);
    let buffered = reader.fill_buf()?;
    let buffered = buffered.to_vec();
    reader.consume(buffered.len());
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
            Start => Some(Marking(b's')),
            Marking(byte) if *byte == b's' => Some(Marking(b'U')),
            Marking(byte) if *byte == b'U' => Some(Marking(b'f')),
            Marking(byte) if *byte == b'f' => Some(Marking(b'F')),
            Marking(byte) if *byte == b'F' => Some(Marking(b'i')),
            Marking(byte) if *byte == b'i' => Some(Marking(b'X')),
            Marking(byte) if *byte == b'X' => Some(End),
            End => None,
            _ => panic!("Unsupported mark"),
        }
    }
    pub fn as_byte(&self) -> Byte {
        match self {
            Marking(v) => *v,
            _ => panic!("Only Marking will have byte value!"),
        }
    }
    pub fn start_mark() -> Marks {
        Marking(b's')
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
        for i in 0..bytes.len() {
            if bytes[i] == start_byte && Start.matches(i, bytes) {
                unmarked.push(&bytes[consumed_segments..i]);
                consumed_segments = i + MARK_SIZE;
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

