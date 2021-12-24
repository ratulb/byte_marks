#![deny(elided_lifetimes_in_paths)]
#![deny(rust_2018_idioms)]
//! ## byte_marks
//!
//! `byte_marks` is a configurable, light weight and intuitive bytes' boundary marker for
//! transmitting and receiving bytes from network/files. The demarcating byte pattern is
//! configured via file called `byte_marks/byte_tail` or a an environment variable named similarly.
//! The characters in the byte mark patterns should not repeat. While trying to demarcate byte//! s its possible that - no progress is being made - its an indication that there may be no
//! byte pattern in the stream being read or default buffer length has been hit without
//! encountering any pattern delimiter.
//!

use lazy_static::lazy_static;
use std::env;
lazy_static! {
    pub static ref MARK: &'static str = Box::leak({
        let markings =
            env::var("byte_mark").unwrap_or_else(|_| include_str!("byte_mark").to_string());
        markings.into_boxed_str()
    });
    pub static ref TAIL: &'static str = Box::leak({
        let tail = env::var("byte_tail").unwrap_or_else(|_| include_str!("byte_tail").to_string());
        tail.into_boxed_str()
    });
}

pub(crate) type Byte = u8;
pub use bytemarker::ByteMarker;
pub use bytemarks::ByteMarks;
pub use marked::Marked;

mod bytemarker;
mod bytemarks;
mod marked;
