# Changelog

## 6.0.0
- Added bounds checking for all IPP string values using a const generic string wrapper `BoundedString<MAX>`
- Switched `IppAttribute` type to use `IppName` aka `BoundedString<255>`.
- Refactored IppOperation construction to return a result ensuring `Uri` values used don't exceed the maximum allowed byte length for IPP URI values.
- Switched `IppAttributeGroup` to use `IppName` in attribute methods to enforce string length contstraints.

## 5.4.0
- Added missing media-col attribute constants
- Updated reqwest dependency to version 0.13
- Make rustls a default backend

## 5.3.2
- Updated to Rust 2024 edition
- Declare IppVersion::vX_X() as const functions

## 5.3.1
- Compilation fix when TLS features are disabled

## 5.3.0
- Added document_format builder function
- Upgraded ureq dependency to version 3
- Added TLS backend selection features to the util project

## 5.2.0
- Added additional attribute constants
- Updated 'thiserror' dependency to version 2

## 5.1.0
- Added `rustls` backend support via two new feature flags: `client-rustls` and `async-client-rustls`
- Only `async-client-tls` feature is enabled by default

## 5.0.5
- Added IPP attributes required by Windows Print Protection mode

## 5.0.4
- Added IppParser::parse_parts function to only read and parse IPP headers without payload
- Added missing `serde` feature to the manifest file
- Added IppRader::into_inner function

## 5.0.3
- Added Eq and Hash derivation for IppValue
- Added into_xx methods for attribute models
- Fixed a bug with unsupported attributes not being sent

## 5.0.2
- Re-added support for `ipp://` scheme in the IPP url

## 5.0.1
- Fixed #22: placing of attribute into incorrect group
- Added missing text-with-language and name-with-language value support

## 5.0.0
- Breaking change: IppValue::Collection now uses BTreeMap instead of Vec
- Added support for custom CA certs in the IPP client builder
- Async print example now uses async IPP payload
- Added more operations to ipputil utility: Purge-Jobs, Cancel-Job, Get-Jobs, Get-Job

## 4.0.0
- Breaking changes in several APIs
- Added blocking client based on `ureq` crate, called `IppClient`
- Renamed asynchronous client to `AsyncIppClient`
- Added basic auth support in the client builder
- Refactored utility functions
- Moved `IppError` into separate module

## 3.0.2

- Added `http_header` method to the client builder which allows to specify a custom HTTP header
- Added `tls` feature

## 3.0.1

- Added IppRequestResponse::into_payload
- Fixed improper ordering of 'printer-uri' attribute

## 3.0.0

- Upgraded tokio dependency to 1.x.
- Added synchronous API (parser and payload) to be used with synchronous HTTP clients.
- Added `async` feature to enable or disable async operations (enabled by default).
- `reqwest` is now the only client  behind the `client` feature.
- Refactored and simplified internal project structure.

## 2.0.0

- Added initial multiclient support, selected via `client-isahc` or `client-reqwest` feature. The default client is `isahc`.
- Use `http::Uri` instead of strings in the APIs. `IppValue::Uri` is still a string though because of parsing and format issues from some
IPP implementations.
- Added `util` module with several high-level utility functions.
- Refactored `IppClientBuilder::new` into `IppClient::builder`.
- Removed `uri` parameter from `IppOperation::into_ipp_request` method.
- Added `From<T: IppOperation>` default implementation for `IppRequestResponse` struct.
- Refactored `IppVersion` enum into a struct.
- Introduced `IppReader` struct for IPP-specific read operations.
- Moved examples into a separate subcrate.
- Several internal fixes and cleanups
- Documentation fixes
