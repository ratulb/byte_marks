# byte_marks
A rust library to mark/unmark transmitted/received byte boundaries for messages.
### [Example](https://github.com/ratulb/byte_marks/blob/main/tests/example.rs)

```rust
        //The segmented message over the wire chunk|suffix| chunk | suffix| tail
        let message = "StreamingsUfFiX withsUfFiX markssUfFiX and tailtAiL";
        //The following is for only showing validation
        let mut segments = Vec::new();
        segments.push("Streaming".as_bytes());
        segments.push(" with".as_bytes());
        segments.push(" marks".as_bytes());
        segments.push(" and tail".as_bytes());
        
        //Cursor is akin to a TcpStream
        let mut cursor = Cursor::new(message.as_bytes());
        
        //From the stream create an iterator and validate the received message chunks
        
        let stream = Marked::new(&mut cursor, "sUfFiX", "tAiL");
        let zipped = stream.into_iter().zip(segments.iter());

        for (unmarked, segment) in zipped {
            assert!(unmarked == segment.to_vec());
        }
        
     
     Shows how to unmark byte chunks out from a stream of marked bytes
     
     let random_texts = [
            "Some random",
            "strings from this array",
            "are getiing",
            "picked and being",
            "converted to",
            "to bytes",
            "and",
            "then suffixed",
            "with byte marks.",
            "These",
            "marked bytes",
            "are then getting",
            "stripped off of",
            "demarcating",
            "pattern",
        ];
 
  let mut randomizer = rand::thread_rng();
        let num_strings: usize = randomizer.gen_range(0..1000);
        let mut orig_strings = vec![];
        let mut marked_bytes = vec![];
        
        let marker = ByteMarker::new("sUfFiX", "");
        
        for _ in 0..num_strings {
            //Pick a random index
            let index = randomizer.gen_range(0..random_texts.len());
            
            //Pick a random string for the given index
            let picked_string = random_texts[index];
            
            //Get the bytes and demarcate with byte marks
            let mut bytes = picked_string.as_bytes().to_vec();
            
            marker.mark_bytes(&mut bytes);//The bytes have been attached with the marker     bytes of ```sUfFiX```
           
           //Preserve the old string for assertion below
            orig_strings.push(picked_string);
           //Keep extending the marked bytes
           marked_bytes.extend(bytes);
        }
        //Marked bytes may be written to a file/sent across the wire
        //Get the orginal strings back from marked bytes on receipt
        
        let unmarked = marker.unmark(&marked_bytes).unwrap().0;
        
        //We must get same bytes back with demarcating bytes removed
        //Lets reconstruct the strings back and validate
        for i in 0..unmarked.len() {
            assert_eq!(
                orig_strings[i],
                String::from_utf8(unmarked[i].to_vec()).unwrap()
            );
        }

```
