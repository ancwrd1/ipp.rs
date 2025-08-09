//!
//! Base IPP definitions and tags
//!
use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use enum_primitive_derive::Primitive;

/// IPP protocol version
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct IppVersion(pub u16);

impl IppVersion {
    pub const fn v1_0() -> Self {
        IppVersion(0x0100)
    }
    pub const fn v1_1() -> Self {
        IppVersion(0x0101)
    }
    pub const fn v2_0() -> Self {
        IppVersion(0x0200)
    }
    pub const fn v2_1() -> Self {
        IppVersion(0x0201)
    }
    pub const fn v2_2() -> Self {
        IppVersion(0x0202)
    }
}

/// IPP operation constants
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
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

    CupsGetDefault = 0x4001,
    CupsGetPrinters = 0x4002,
    CupsAddModifyPrinter = 0x4003,
    CupsDeletePrinter = 0x4004,
    CupsGetClasses = 0x4005,
    CupsAddModifyClass = 0x4006,
    CupsDeleteClass = 0x4007,
    CupsAcceptJobs = 0x4008,
    CupsRejectJobs = 0x4009,
    CupsSetDefault = 0x400A,
    CupsGetDevices = 0x400B,
    CupsGetPPDs = 0x400C,
    CupsMoveJob = 0x400D,
    CupsAuthenticateJob = 0x400E,
    CupsGetPPD = 0x400F,
    CupsGetDocument = 0x4027,
    CupsCreateLocalPrinter = 0x4028,
}

/// printer-state constants
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrinterState {
    Idle = 3,
    Processing = 4,
    Stopped = 5,
}

/// paper orientation constants
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Orientation {
    Portrait = 3,
    Landscape = 4,
    ReverseLandscape = 5,
    ReversePortrait = 6,
}

/// print-quality constants
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrintQuality {
    Draft = 3,
    Normal = 4,
    High = 5,
}

/// finishings constants
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum Finishings {
    None = 3,
    Staple = 4,
    Punch = 5,
    Cover = 6,
    Bind = 7,
    SaddleStitch = 8,
    EdgeStitch = 9,
    StapleTopLeft = 20,
    StapleBottomLeft = 21,
    StapleTopRight = 22,
    StapleBottomRight = 23,
    EdgeStitchLeft = 24,
    EdgeStitchTop = 25,
    EdgeStitchRight = 26,
    EdgeStitchBottom = 27,
    StapleDualLeft = 28,
    StapleDualTop = 29,
    StapleDualRight = 30,
    StapleDualBottom = 31,
    StapleTripleLeft = 32,
    StapleTripleTop = 33,
    StapleTripleRight = 34,
    StapleTripleBottom = 35,
    PunchTopLeft = 70,
    PunchBottomLeft = 71,
    PunchTopRight = 72,
    PunchBottomRight = 73,
    PunchDualLeft = 74,
    PunchDualTop = 75,
    PunchDualRight = 76,
    PunchDualBottom = 77,
    PunchTripleLeft = 78,
    PunchTripleTop = 79,
    PunchTripleRight = 80,
    PunchTripleBottom = 81,
    PunchQuadLeft = 82,
    PunchQuadTop = 83,
    PunchQuadRight = 84,
    PunchQuadBottom = 85,
}

/// job-state constants
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum JobState {
    Pending = 3,
    PendingHeld = 4,
    Processing = 5,
    ProcessingStopped = 6,
    Canceled = 7,
    Aborted = 8,
    Completed = 9,
}

/// group delimiter tags
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Primitive, Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum DelimiterTag {
    OperationAttributes = 0x01,
    JobAttributes = 0x02,
    EndOfAttributes = 0x03,
    PrinterAttributes = 0x04,
    UnsupportedAttributes = 0x05,
}

/// IPP value tags
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ValueTag {
    Unsupported = 0x10,
    Unknown = 0x12,
    NoValue = 0x13,
    Integer = 0x21,
    Boolean = 0x22,
    Enum = 0x23,
    OctetStringUnspecified = 0x30,
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

/// IPP status codes
#[derive(Primitive, Debug, Copy, Clone, Eq, PartialEq)]
pub enum StatusCode {
    SuccessfulOk = 0x0000,
    SuccessfulOkIgnoredOrSubstitutedAttributes = 0x0001,
    SuccessfulOkConflictingAttributes = 0x0002,
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
    UnknownStatusCode = 0xffff,
}

impl StatusCode {
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            StatusCode::SuccessfulOk
                | StatusCode::SuccessfulOkIgnoredOrSubstitutedAttributes
                | StatusCode::SuccessfulOkConflictingAttributes
        )
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatusCode::SuccessfulOk => write!(f, "No error"),
            StatusCode::SuccessfulOkIgnoredOrSubstitutedAttributes => write!(f, "Ignored or substituted attributes"),
            StatusCode::SuccessfulOkConflictingAttributes => write!(f, "Conflicting attributes"),
            StatusCode::ClientErrorBadRequest => write!(f, "Bad request"),
            StatusCode::ClientErrorForbidden => write!(f, "Forbidden"),
            StatusCode::ClientErrorNotAuthenticated => write!(f, "Not authenticated"),
            StatusCode::ClientErrorNotAuthorized => write!(f, "Not authorized"),
            StatusCode::ClientErrorNotPossible => write!(f, "Not possible"),
            StatusCode::ClientErrorTimeout => write!(f, "Timeout"),
            StatusCode::ClientErrorNotFound => write!(f, "Not found"),
            StatusCode::ClientErrorGone => write!(f, "Gone"),
            StatusCode::ClientErrorRequestEntityTooLong => write!(f, "Entity too long"),
            StatusCode::ClientErrorRequestValueTooLong => write!(f, "Request value too long"),
            StatusCode::ClientErrorDocumentFormatNotSupported => write!(f, "Document format not supported"),
            StatusCode::ClientErrorAttributesOrValuesNotSupported => write!(f, "Attributes or values not supported"),
            StatusCode::ClientErrorUriSchemeNotSupported => write!(f, "Uri scheme not supported"),
            StatusCode::ClientErrorCharsetNotSupported => write!(f, "Charset not supported"),
            StatusCode::ClientErrorConflictingAttributes => write!(f, "Conflicting attributes"),
            StatusCode::ClientErrorCompressionNotSupported => write!(f, "Compression not supported"),
            StatusCode::ClientErrorCompressionError => write!(f, "Compression error"),
            StatusCode::ClientErrorDocumentFormatError => write!(f, "Document format error"),
            StatusCode::ClientErrorDocumentAccessError => write!(f, "Document access error"),
            StatusCode::ServerErrorInternalError => write!(f, "Internal error"),
            StatusCode::ServerErrorOperationNotSupported => write!(f, "Operation not supported"),
            StatusCode::ServerErrorServiceUnavailable => write!(f, "Service unavailable"),
            StatusCode::ServerErrorVersionNotSupported => write!(f, "Version not supported"),
            StatusCode::ServerErrorDeviceError => write!(f, "Device error"),
            StatusCode::ServerErrorTemporaryError => write!(f, "Temporary error"),
            StatusCode::ServerErrorNotAcceptingJobs => write!(f, "Not accepting jobs"),
            StatusCode::ServerErrorBusy => write!(f, "Busy"),
            StatusCode::ServerErrorJobCanceled => write!(f, "Job canceled"),
            StatusCode::ServerErrorMultipleDocumentJobsNotSupported => {
                write!(f, "Multiple document jobs not supported")
            }
            StatusCode::UnknownStatusCode => write!(f, "Unknown status code"),
        }
    }
}
