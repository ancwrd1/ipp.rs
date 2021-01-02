# Changelog

## 3.0.0

- Upgraded to tokio 1.0
- `request` is now a default client

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
