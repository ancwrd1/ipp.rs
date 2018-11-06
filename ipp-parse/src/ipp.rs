//!
//! Base IPP definitions and tags
//!
use std::fmt;

use enum_primitive_derive::Primitive;

#[derive(Debug, Primitive)]
pub enum Operation {
    PrintJob = 0x0002,
    PrintUri = 0x0003,
    ValidateJob = 0x0004,
    CreateJob = 0x0005,
    SendDocument = 0x0006,
    SendUri = 0x0007,
    CancelJob = 0x0008,
    GetJobAttributes = 0x0009,
    GetJobs = 0x000A,
    GetPrinterAttributes = 0x000B,
    HoldJob = 0x000C,
    ReleaseJob = 0x000D,
    RestartJob = 0x000E,
    PausePrinter = 0x0010,
    ResumePrinter = 0x0011,
    PurgeJobs = 0x0012,
}

#[derive(Primitive, Debug, Copy, Clone, PartialEq)]
pub enum PrinterState {
    Idle = 3,
    Processing = 4,
    Stopped = 5,
}

#[derive(Primitive, Debug, Copy, Clone, PartialEq)]
pub enum Orientation {
    Portrait = 3,
    Landscape = 4,
    ReverseLandscape = 5,
    ReversePortrait = 6,
}

#[derive(Primitive, Debug, Copy, Clone, PartialEq)]
pub enum PrintQuality {
    Draft = 3,
    Normal = 4,
    High = 5,
}

#[derive(Primitive, Debug, Copy, Clone, PartialEq)]
pub enum Finishings {
    None = 3,
    Staple = 4,
    Punch = 5,
    Cover = 6,
    Bind = 7,
    SaddleStitch = 8,
    EdgeStitch = 9,
}

#[derive(Primitive)]
pub enum JobState {
    Pending = 3,
    PendingHeld = 4,
    Processing = 5,
    ProcessingStopped = 6,
    Canceled = 7,
    Aborted = 8,
    Completed = 9,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Primitive)]
pub enum DelimiterTag {
    OperationAttributes = 0x01,
    JobAttributes = 0x02,
    EndOfAttributes = 0x03,
    PrinterAttributes = 0x04,
    UnsupportedAttributes = 0x05,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Primitive)]
pub enum ValueTag {
    Unsupported = 0x10,
    Unknown = 0x12,
    NoValue = 0x13,
    Integer = 0x21,
    Boolean = 0x22,
    Enum = 0x23,
    OctectStringUnspecified = 0x30,
    DateTime = 0x31,
    Resolution = 0x32,
    RangeOfInteger = 0x33,
    BegCollection = 0x34,
    TextWithLanguage = 0x35,
    NameWithLanguage = 0x36,
    EndCollection = 0x37,
    TextWithoutLanguage = 0x41,
    NameWithoutLanguage = 0x42,
    Keyword = 0x44,
    Uri = 0x45,
    UriScheme = 0x46,
    Charset = 0x47,
    NaturalLanguage = 0x48,
    MimeMediaType = 0x49,
    MemberAttrName = 0x4a,
}

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
