# Respirator - RESP parser in Rust

Respirator is [nom](https://github.com/Geal/nom) based [Redis Serialization Protocol (resp)](https://redis.io/topics/protocol) parser. Currently only "complete" parsing (i.e. works only when all data to parse is available) works, yet an aim is to cover streaming parsing as well.

## Example
```
use respirator::{Resp, resp};
use std::matches;

let input = &b"*2\r\n$2\r\nOK\r\n$4\r\nResp\r\n"[..];
let (_, parsed) = resp(input).unwrap();
if let Resp::Array(Some(values)) = parsed {
  let simple_string = &values[0];
  let bulk_string = &values[1];
  assert!(matches!(Resp::SimpleString(b"OK".to_vec()), simple_string));
  assert!(matches!(Resp::BulkString(Some(b"Resp".to_vec())), bulk_string));
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
