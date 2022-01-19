//! ## Marked
//!

use crate::{Byte, ByteMarks, MARK, TAIL};
use std::io::BufRead;

pub struct Marked<'a, R>
where
    R: BufRead,
{
    reader: &'a mut R,
    initializer: ByteMarks<'a>,
    marks: ByteMarks<'a>,
    tail: Option<ByteMarks<'a>>,
    bytes_fetched: usize,
    curr_buf: Option<Vec<Byte>>,
    left_over: Option<Vec<Byte>>,
    buf_pos: usize,
    eof_reached: bool,
    start_byte: Byte,
    mark_size: usize,
    tail_start_byte: Byte,
    tail_size: usize,
}

impl<'a, R> Marked<'a, R>
where
    R: BufRead,
{
    pub fn with_defaults(r: &'a mut R) -> Self {
        Self::new(r, &MARK, &TAIL)
    }
    pub fn new(r: &'a mut R, mark: &'a str, tail: &'a str) -> Self {
        let initializer = ByteMarks::initialize(mark, tail);
        let marks = initializer.init_marking_indices();
        let tail = if initializer.tail_bytes_len() > 0 {
            Some(initializer.init_tail_indices())
        } else {
            None
        };

        let start_byte = initializer.marking_start_byte();
        let mark_size = initializer.marking_bytes_len();
        let (tail_start_byte, tail_size) = if tail.is_some() {
            (initializer.tail_start_byte(), initializer.tail_bytes_len())
        } else {
            (0, 0)
        };
        Self {
            reader: r,
            initializer,
            marks,
            tail,
            bytes_fetched: 0,
            curr_buf: None,
            left_over: None,
            buf_pos: 0,
            eof_reached: false,
            start_byte,
            mark_size,
            tail_start_byte,
            tail_size,
        }
    }
}

impl<'a, R> Iterator for Marked<'a, R>
where
    R: BufRead,
{
    type Item = Vec<Byte>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.eof_reached {
                return None;
            }
            match self.curr_buf {
                Some(ref bytes) => {
                    let mut index = self.buf_pos;
                    while index < bytes.len() {
                        if bytes[index] == self.start_byte
                            && self.initializer.marking_matches(
                                &self.initializer,
                                &self.marks,
                                index,
                                bytes,
                            )
                        {
                            let next = Some(bytes[self.buf_pos..index].to_vec());
                            self.buf_pos = index + self.mark_size;
                            if self.buf_pos == bytes.len() {
                                self.curr_buf = None;
                            }
                            return next;
                        }

                        if let Some(ref tail) = self.tail {
                            if bytes[index] == self.tail_start_byte
                                && self.initializer.tail_marking_matches(
                                    &self.initializer,
                                    tail,
                                    index,
                                    bytes,
                                )
                            {
                                let pre_tail = Some(bytes[self.buf_pos..index].to_vec());
                                self.buf_pos = index + self.tail_size;
                                self.eof_reached = true;
                                self.curr_buf = None;
                                return pre_tail;
                            }
                        }
                        index += 1;
                    }

                    if index == bytes.len() {
                        self.left_over = Some(bytes[self.buf_pos..].to_vec());
                        self.curr_buf = None;
                    }
                }
                None => match self.reader.fill_buf() {
                    Ok(buf) if buf.is_empty() => {
                        return self.left_over.take();
                    }
                    Ok(buf) => {
                        self.bytes_fetched += buf.len();
                        self.buf_pos = 0;
                        if let Some(mut left_over) = self.left_over.take() {
                            left_over.extend(buf);
                            self.curr_buf = Some(left_over);
                        } else {
                            self.curr_buf = Some(buf.to_vec());
                        }
                        self.reader.consume(self.bytes_fetched);
                    }
                    Err(err) => {
                        eprintln!("Error filling buf = {:?}", err);
                        return None;
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Cursor;
    use std::io::{BufReader, Result};

    #[test]
    #[should_panic]
    fn test_file_with_multiple_marks_with_mark_and_should_panic() {
        let mut reader =
            BufReader::new(File::open("tests/file_with_multiple_marks_at_end.txt").unwrap());
        let marked = Marked::new(&mut reader, "sUfFiX", "");
        let content = "tests/file_with_multiple_marks_at_end.txt";
        for bytes in marked {
            let s = String::from_utf8(bytes).unwrap();
            assert_eq!(content, &s);
        }
    }
    #[test]
    fn test_file_with_multiple_marks_with_mark_and_panic_handled() {
        let mut reader =
            BufReader::new(File::open("tests/file_with_multiple_marks_at_end.txt").unwrap());
        let marked = Marked::new(&mut reader, "sUfFiX", "");
        let content = "tests/file_with_multiple_marks_at_end.txt";
        let mut first = true;
        for bytes in marked {
            let mut s = String::from_utf8(bytes).unwrap();
            if first {
                //Since we are passing 'sUfFiX' first line would have no preceding '\n'
                first = false;
            } else if s.starts_with('\n') {
                //2nd line onwards will have preceding '\n'
                s.drain(0..1);
            }
            if s == String::from("") {
                continue; //This is because last line would be only '\n' - since we are taking out above
            }
            assert_eq!(content, &s);
        }
    }

    #[test]
    fn test_empty_reader() {
        let mut cursor = Cursor::new(Vec::new());
        let marked = Marked::with_defaults(&mut cursor);
        for bytes in marked {
            assert!(!bytes.is_empty());
        }
    }
    #[test]
    fn test_empty_file() -> Result<()> {
        let mut reader = BufReader::new(File::open("tests/empty.txt")?);
        let marked = Marked::with_defaults(&mut reader);
        for bytes in marked {
            assert!(!bytes.is_empty());
        }
        Ok(())
    }
    #[test]
    fn test_file_random_content() -> Result<()> {
        let mut reader = BufReader::new(File::open("tests/random.txt")?);
        let marked = Marked::with_defaults(&mut reader);
        for bytes in marked {
            assert!(!bytes.is_empty());
        }
        Ok(())
    }
    #[test]
    fn test_file_with_mark_at_end() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let mut reader = BufReader::new(File::open("tests/file_with_mark_at_end.txt")?);
        let marked = Marked::with_defaults(&mut reader);
        let content = "tests/file_with_mark_at_end.txt";
        for bytes in marked {
            assert_eq!(content, &String::from_utf8(bytes)?);
        }
        Ok(())
    }
    #[test]
    fn test_file_with_multiple_marks_at_end() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let mut reader = BufReader::new(File::open("tests/file_with_multiple_marks_at_end.txt")?);
        let marked = Marked::with_defaults(&mut reader);
        let content = "tests/file_with_multiple_marks_at_end.txt";
        for bytes in marked {
            println!("asserted");
            assert_eq!(content, &String::from_utf8(bytes)?);
        }
        Ok(())
    }
    #[test]
    fn test_file_with_multiple_marks_with_defaults() {
        let mut reader =
            BufReader::new(File::open("tests/file_with_multiple_marks_at_end.txt").unwrap());
        let marked = Marked::with_defaults(&mut reader);
        let content = "tests/file_with_multiple_marks_at_end.txt";
        for bytes in marked {
            assert_eq!(content, &String::from_utf8(bytes).unwrap());
        }
    }

    #[test]
    fn test_reader_with_reduced_capacity() -> Result<()> {
        let mut reader = BufReader::with_capacity(1, File::open("tests/random.txt")?);
        let marked = Marked::with_defaults(&mut reader);
        for bytes in marked {
            assert!(!bytes.is_empty());
        }
        Ok(())
    }

    #[test]
    fn test_not_empty_reader() {
        let mut cursor = Cursor::new(vec![1, 2, 3]);
        let marked = Marked::with_defaults(&mut cursor);
        for bytes in marked {
            assert!(!bytes.is_empty());
        }
    }
    #[test]
    fn test_stream_with_marks_and_tail() {
        let message =
            "This issUfFiX a msgsUfFiX with interspercedsUfFiX with suffixes and finally atAiL";
        let mut segments = Vec::new();
        segments.push("This is".as_bytes());
        segments.push(" a msg".as_bytes());
        segments.push(" with intersperced".as_bytes());
        segments.push(" with suffixes and finally a".as_bytes());

        let mut cursor = Cursor::new(message.as_bytes());
        let marked = Marked::new(&mut cursor, "sUfFiX", "tAiL");
        let zipped = marked.into_iter().zip(segments.iter());

        for (unmarked, segment) in zipped {
            assert!(unmarked == segment.to_vec());
        }
    }
}



