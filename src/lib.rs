#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(elided_lifetimes_in_paths)]
#![deny(rust_2018_idioms)]
//! ## byte_marks
//!
//! `byte_marks` is a configurable, light weight and intuitive bytes' boundary marker for
//! transmitting and receiving bytes from network/files. This comes very handy while building
//! application network protocols - one could read off the demarcated bytes of the wire and
//! could use bincode <https://github.com/bincode-org/bincode> to reconstruct a struct from those bytes. The demarcating byte pattern
//! is configured via files called `byte_mark/byte_tail` or environment variables named
//! similarly and or in code. The characters in the pattern should not repeat.
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
