#[cfg(test)]
mod tests {
    use byte_marks::Marks;
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

    #[test]
    fn marks_unmark_test_1() -> Result<()> {
        let f = "msg_empty.txt";
        let f = File::open(f)?;
        let mut reader = BufReader::new(f);
        let mut consumed = 0;
        loop {
            reader.consume(consumed);
            let buf = reader.fill_buf()?;
            if buf.len() == 0 {
                break;
            }
            let unmarked = Marks::unmark(&buf);

            if let Some(unmarked) = unmarked {
                for i in 0..unmarked.0.len() {
                    println!("The msg = {:?}", String::from_utf8(unmarked.0[i].to_vec()));
                }
                println!("Bytes consumed = {:?}", unmarked.1);
                consumed += unmarked.1;
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

    #[cfg(feature = "default_async")]
    async fn default_async_buffered_bytes(f: &str) -> OtherResult<Vec<u8>> {
        let f = OtherFile::open(f).await?;
        let mut reader = OtherBufReader::new(f);
        let buffered = reader.fill_buf().await?;
        let buffered = buffered.to_vec();
        //reader.consume_unpin(buffered.len());
        Ok(buffered)
    }
}
