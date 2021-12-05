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

    #[test]
    fn unmark_one_msg_std_test_1() -> Result<()> {
        let f = "tests/single-msg.txt";
        let f = File::open(f)?;
        let mut reader = BufReader::new(f);
        let mut consumed = 0;
        loop {
            reader.consume(consumed);
            let buf = reader.fill_buf()?;
            if buf.len() == 0 {
                break;
            }
            //Following is the content inside the file with 'oNe-MsG' as content at the end
            //'This is one message till the delimiter at the endoNe-MsG'
            std::env::set_var("byte_marks", "oNe-MsG");
            let unmarked = Marks::unmark(&buf);

            if let Some(unmarked) = unmarked {
                for i in 0..unmarked.0.len() {
                    assert_eq!(
                        "This is one message till the delimiter at the end",
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
    async fn unmark_one_msg_async_std_test_1() -> OtherResult<()> {
        let f = "tests/single-msg.txt";
        let f = OtherFile::open(f).await?;
        let mut reader = OtherBufReader::new(f);
        let mut consumed = 0;
        loop {
            reader.consume_unpin(consumed);
            let buf = reader.fill_buf().await?;
            if buf.len() == 0 {
                break;
            }
            //Following is the content inside the file with 'oNe-MsG' as content at the end
            //'This is one message till the delimiter at the endoNe-MsG'
            std::env::set_var("byte_marks", "oNe-MsG");
            let unmarked = Marks::unmark(&buf);

            if let Some(unmarked) = unmarked {
                for i in 0..unmarked.0.len() {
                    assert_eq!(
                        "This is one message till the delimiter at the end",
                        String::from_utf8(unmarked.0[i].to_vec()).unwrap()
                    );
                }
                consumed += unmarked.1;
            }
        }
        Ok(())
    }
}
