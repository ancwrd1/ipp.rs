//!
//! IPP attribute names
//!

// enum_primitive only works with integer values, so these will have to remain as constants

pub const ATTRIBUTES_CHARSET: &str = "attributes-charset";
pub const ATTRIBUTES_NATURAL_LANGUAGE: &str = "attributes-natural-language";
pub const CHARSET_CONFIGURED: &str = "charset-configured";
pub const CHARSET_SUPPORTED: &str = "charset-supported";
pub const COMPRESSION_SUPPORTED: &str = "compression-supported";
pub const DOCUMENT_FORMAT_DEFAULT: &str = "document-format-default";
pub const DOCUMENT_FORMAT_SUPPORTED: &str = "document-format-supported";
pub const GENERATED_NATURAL_LANGUAGE_SUPPORTED: &str = "generated-natural-language-supported";
pub const IPP_VERSIONS_SUPPORTED: &str = "ipp-versions-supported";
pub const NATURAL_LANGUAGE_CONFIGURED: &str = "natural-language-configured";
pub const OPERATIONS_SUPPORTED: &str = "operations-supported";
pub const PDL_OVERRIDE_SUPPORTED: &str = "pdl-override-supported";
pub const PRINTER_IS_ACCEPTING_JOBS: &str = "printer-is-accepting-jobs";
pub const PRINTER_MAKE_AND_MODEL: &str = "printer-make-and-model";
pub const PRINTER_NAME: &str = "printer-name";
pub const PRINTER_STATE: &str = "printer-state";
pub const PRINTER_STATE_MESSAGE: &str = "printer-state-message";
pub const PRINTER_STATE_REASONS: &str = "printer-state-reasons";
pub const PRINTER_UP_TIME: &str = "printer-up-time";
pub const PRINTER_URI: &str = "printer-uri";
pub const PRINTER_URI_SUPPORTED: &str = "printer-uri-supported";
pub const QUEUED_JOB_COUNT: &str = "queued-job-count";
pub const URI_AUTHENTICATION_SUPPORTED: &str = "uri-authentication-supported";
pub const URI_SECURITY_SUPPORTED: &str = "uri-security-supported";
pub const JOB_ID: &str = "job-id";
pub const JOB_NAME: &str = "job-name";
pub const JOB_STATE: &str = "job-state";
pub const JOB_STATE_REASONS: &str = "job-state-reasons";
pub const JOB_URI: &str = "job-uri";
pub const LAST_DOCUMENT: &str = "last-document";
pub const REQUESTING_USER_NAME: &str = "requesting-user-name";
pub const STATUS_MESSAGE: &str = "status-message";
pub const REQUESTED_ATTRIBUTES: &str = "requested-attributes";
pub const SIDES_SUPPORTED: &str = "sides-supported";
pub const OUTPUT_MODE_SUPPORTED: &str = "output-mode-supported";
pub const COLOR_SUPPORTED: &str = "color-supported";
pub const PRINTER_INFO: &str = "printer-info";
pub const PRINTER_LOCATION: &str = "printer-location";
pub const PRINTER_MORE_INFO: &str = "printer-more-info";
pub const PRINTER_RESOLUTION_DEFAULT: &str = "printer-resolution-default";
pub const PRINTER_RESOLUTION_SUPPORTED: &str = "printer-resolution-supported";
pub const COPIES_SUPPORTED: &str = "copies-supported";
pub const COPIES_DEFAULT: &str = "copies-default";
pub const SIDES_DEFAULT: &str = "sides-default";
pub const PRINT_QUALITY_DEFAULT: &str = "print-quality-default";
pub const PRINT_QUALITY_SUPPORTED: &str = "print-quality-supported";
pub const FINISHINGS_DEFAULT: &str = "finishings-default";
pub const FINISHINGS_SUPPORTED: &str = "finishings-supported";
pub const OUTPUT_BIN_DEFAULT: &str = "output-bin-default";
pub const OUTPUT_BIN_SUPPORTED: &str = "output-bin-supported";
pub const ORIENTATION_REQUESTED_DEFAULT: &str = "orientation-requested-default";
pub const ORIENTATION_REQUESTED_SUPPORTED: &str = "orientation-requested-supported";
pub const MEDIA_DEFAULT: &str = "media-default";
pub const MEDIA_SUPPORTED: &str = "media-supported";
pub const PAGES_PER_MINUTE: &str = "pages-per-minute";
pub const COLOR_MODE_SUPPORTED: &str = "color-mode-supported";
pub const PRINT_COLOR_MODE_SUPPORTED: &str = "print-color-mode-supported";

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
