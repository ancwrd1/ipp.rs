//!
//! IPP status codes
//!

use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Primitive)]
pub enum StatusCode {
    SuccessfulOK = 0x0000,
    SuccessfulOKIgnoredOrSubstitutedAttributes = 0x0001,
    SuccessfulOKConflictingAttributes = 0x0002,
    ClientErrorBadRequest = 0x0400,
    ClientErrorForbidden = 0x0401,
    ClientErrorNotAuthenticated = 0x0402,
    ClientErrorNotAuthorized = 0x0403,
    ClientErrorNotPossible = 0x0404,
    ClientErrorTimeout = 0x0405,
    ClientErrorNotFound = 0x0406,
    ClientErrorGone = 0x0407,
    ClientErrorRequestEntityTooLong = 0x0408,
    ClientErrorRequestValueTooLong = 0x0409,
    ClientErrorDocumentFormatNotSupported = 0x040A,
    ClientErrorAttributesOrValuesNotSupported = 0x040B,
    ClientErrorUriSchemeNotSupported = 0x040C,
    ClientErrorCharsetNotSupported = 0x040D,
    ClientErrorConflictingAttributes = 0x040E,
    ClientErrorCompressionNotSupported = 0x040F,
    ClientErrorCompressionError = 0x0410,
    ClientErrorDocumentFormatError = 0x0411,
    ClientErrorDocumentAccessError = 0x0412,
    ServerErrorInternalError = 0x0500,
    ServerErrorOperationNotSupported = 0x0501,
    ServerErrorServiceUnavailable = 0x0502,
    ServerErrorVersionNotSupported = 0x0503,
    ServerErrorDeviceError = 0x0504,
    ServerErrorTemporaryError = 0x0505,
    ServerErrorNotAcceptingJobs = 0x0506,
    ServerErrorBusy = 0x0507,
    ServerErrorJobCanceled = 0x0508,
    ServerErrorMultipleDocumentJobsNotSupported = 0x0509,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:0x}", *self as u16)
    }
}
