# Changelog

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
