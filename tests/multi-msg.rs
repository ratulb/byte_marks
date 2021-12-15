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
        let mut reader = BufReader::with_capacity(64, f);
        //let mut reader = BufReader::new(f);
        let mut consumed = 0;
        let mut first = true;
        let mut looped = 0;
        let mut left_over: Option<&[u8]> = None;
        let mut combined = vec![];
        loop {
            if first == false {
                if looped > 0 && consumed == 0 {
                    break;
                }
                reader.consume(consumed);
                println!("Set consumed = {:?}", consumed);
            } else {
                first = false;
            }
            let buf = reader.fill_buf()?;
            if buf.len() == 0 {
                break;
            }
            consumed += buf.len();
            combined = match left_over.take() {
                Some(remained) => Marks::concat_u8(remained, buf),
                None => buf.to_vec(),
            };
            let unmarked = Marks::unmark(&combined);
            //Following is the content inside the file with 'sUfFiX' as content at the end
            //'Just a testsUfFiX'

            left_over = match unmarked {
                Some((untagged, remained)) => {
                    for i in 0..untagged.len() {
                        assert_eq!(
                            "Just a test",
                            String::from_utf8(untagged[i].to_vec()).unwrap()
                        );
                    }
                    remained
                }
                None => None,
            };

            looped += 1;
        }
        println!("Consumed = {:?}", consumed);
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
