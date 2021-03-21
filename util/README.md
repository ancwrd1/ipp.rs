## IPP command line utility

This crate contains `ipputil` CLI application to perform common IPP operations
such as getting printer attributes or printing a file.

To install it run `cargo install ipp-util`. For command line use usage run `ipputil --help`.

Usage example:

```
ipputil print -f /path/to/file.pdf http://192.168.1.100:631/ipp
```

## License

Licensed under MIT or Apache license ([LICENSE-MIT](https://opensource.org/licenses/MIT) or [LICENSE-APACHE](https://opensource.org/licenses/Apache-2.0))
