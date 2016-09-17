IPP protocol implementation for Rust

Usage example:

```rust
pub fn main() {
	let mut req = IppRequest::new(GET_PRINTER_ATTRIBUTES, "http://localhost:631/printers/test-printer");
	let client = IppClient::new();
	let attrs = client.send(&mut req).unwrap();
	for (_, v) in attrs.get_group(PRINTER_ATTRIBUTES_TAG).unwrap() {
    	println!("{}: {}", v.name(), v.value());
	}
}
```
