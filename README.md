# byte_marks
A rust library to mark/unmark transmitted/received byte boundaries for messages.
### [Example](https://github.com/ratulb/byte_marks/blob/main/tests/random-msg.rs)

```rust
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
        for _ in 0..num_strings {
            //Pick a random index
            let index = randomizer.gen_range(0..random_texts.len());
            //Pick a random string for the given index
            let picked_string = random_texts[index];
            //Get the bytes and demarcate with byte marks
            let mut bytes = picked_string.as_bytes().to_vec();
            Marks::mark_bytes(&mut bytes);//The bytes have been demarcated now
            //Preserve the old string for assertion below
            orig_strings.push(picked_string);
           //Keep extending the marked bytes
           marked_bytes.extend(bytes);
        }
        //Marked bytes may be written to a file/sent across the wire
        //Get the orginal strings back from marked bytes on receipt
        let unmarked = Marks::unmark(&marked_bytes).unwrap().0;
        //We must get same bytes back with demarcating bytes removed
        //Lets reconstruct the strings back and validate
        for i in 0..unmarked.len() {
            assert_eq!(
                orig_strings[i],
                String::from_utf8(unmarked[i].to_vec()).unwrap()
            );
        }

```
