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
            io::{BufReader as OtherBufReader, Result as OtherResult},
        },
        futures::io::AsyncBufReadExt,
    };
    #[cfg(feature = "std")]
    #[test]
    fn unmark_multi_msg_std_test_1() -> Result<()> {
        let f = "tests/multi-msg.txt";
        let f = File::open(f)?;
        let mut reader = BufReader::new(f);
        let mut consumed = 0;
        loop {
            reader.consume(consumed);
            let buf = reader.fill_buf()?;
            if buf.len() == 0 {
                break;
            }
            //Following is the content inside the file with 'sUfFiX' as content at the end
            //'Just a testsUfFiX'
            let unmarked = Marks::unmark(&buf);

            if let Some(unmarked) = unmarked {
                for i in 0..unmarked.0.len() {
                    assert_eq!(
                        "Just a test",
                        String::from_utf8(unmarked.0[i].to_vec()).unwrap()
                    );
                }
                consumed += unmarked.1;
            }
        }
        Ok(())
    }
    #[cfg(feature = "default_async")]
    #[async_std::test]
    async fn unmark_multi_msg_async_std_test_1() -> OtherResult<()> {
        let f = "tests/multi-msg.txt";
        let f = OtherFile::open(f).await?;
        let mut reader = OtherBufReader::new(f);
        let mut consumed = 0;
        loop {
            reader.consume_unpin(consumed);
            let buf = reader.fill_buf().await?;
            if buf.len() == 0 {
                break;
            }
            //Following is the content inside the file with 'sUfFiX' as content at the end
            //'Just a testsUfFiX'
            let unmarked = Marks::unmark(&buf);

            if let Some(unmarked) = unmarked {
                for i in 0..unmarked.0.len() {
                    assert_eq!(
                        "Just a test",
                        String::from_utf8(unmarked.0[i].to_vec()).unwrap()
                    );
                }
                consumed += unmarked.1;
            }
        }
        Ok(())
    }
}
