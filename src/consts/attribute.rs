//!
//! IPP attribute names
//!

// enum_primitive only works with integer values, so these will have to remain as constants

pub const ATTRIBUTES_CHARSET: &'static str = "attributes-charset";
pub const ATTRIBUTES_NATURAL_LANGUAGE: &'static str = "attributes-natural-language";
pub const CHARSET_CONFIGURED: &'static str = "charset-configured";
pub const CHARSET_SUPPORTED: &'static str = "charset-supported";
pub const COMPRESSION_SUPPORTED: &'static str = "compression-supported";
pub const DOCUMENT_FORMAT_DEFAULT: &'static str = "document-format-default";
pub const DOCUMENT_FORMAT_SUPPORTED: &'static str = "document-format-supported";
pub const GENERATED_NATURAL_LANGUAGE_SUPPORTED: &'static str = "generated-natural-language-supported";
pub const IPP_VERSIONS_SUPPORTED: &'static str = "ipp-versions-supported";
pub const NATURAL_LANGUAGE_CONFIGURED: &'static str = "natural-language-configured";
pub const OPERATIONS_SUPPORTED: &'static str = "operations-supported";
pub const PDL_OVERRIDE_SUPPORTED: &'static str = "pdl-override-supported";
pub const PRINTER_IS_ACCEPTING_JOBS: &'static str = "printer-is-accepting-jobs";
pub const PRINTER_MAKE_AND_MODEL: &'static str = "printer-make-and-model";
pub const PRINTER_NAME: &'static str = "printer-name";
pub const PRINTER_STATE: &'static str = "printer-state";
pub const PRINTER_STATE_MESSAGE: &'static str = "printer-state-message";
pub const PRINTER_STATE_REASONS: &'static str = "printer-state-reasons";
pub const PRINTER_UP_TIME: &'static str = "printer-up-time";
pub const PRINTER_URI: &'static str = "printer-uri";
pub const PRINTER_URI_SUPPORTED: &'static str = "printer-uri-supported";
pub const QUEUED_JOB_COUNT: &'static str = "queued-job-count";
pub const URI_AUTHENTICATION_SUPPORTED: &'static str = "uri-authentication-supported";
pub const URI_SECURITY_SUPPORTED: &'static str = "uri-security-supported";
pub const JOB_ID: &'static str = "job-id";
pub const JOB_NAME: &'static str = "job-name";
pub const JOB_STATE: &'static str = "job-state";
pub const JOB_STATE_REASONS: &'static str = "job-state-reasons";
pub const JOB_URI: &'static str = "job-uri";
pub const LAST_DOCUMENT: &'static str = "last-document";
pub const REQUESTING_USER_NAME: &'static str = "requesting-user-name";
pub const STATUS_MESSAGE: &'static str = "status-message";
pub const REQUESTED_ATTRIBUTES: &'static str = "requested-attributes";
pub const SIDES_SUPPORTED: &'static str = "sides-supported";
pub const OUTPUT_MODE_SUPPORTED: &'static str = "output-mode-supported";
pub const COLOR_SUPPORTED: &'static str = "color-supported";
pub const PRINTER_INFO: &'static str = "printer-info";
pub const PRINTER_LOCATION: &'static str = "printer-location";
pub const PRINTER_MORE_INFO: &'static str = "printer-more-info";
pub const PRINTER_RESOLUTION_DEFAULT: &'static str = "printer-resolution-default";
pub const PRINTER_RESOLUTION_SUPPORTED: &'static str = "printer-resolution-supported";
pub const COPIES_SUPPORTED: &'static str = "copies-supported";
pub const COPIES_DEFAULT: &'static str = "copies-default";
pub const SIDES_DEFAULT: &'static str = "sides-default";
pub const PRINT_QUALITY_DEFAULT: &'static str = "print-quality-default";
pub const PRINT_QUALITY_SUPPORTED: &'static str = "print-quality-supported";
pub const FINISHINGS_DEFAULT: &'static str = "finishings-default";
pub const FINISHINGS_SUPPORTED: &'static str = "finishings-supported";
pub const OUTPUT_BIN_DEFAULT: &'static str = "output-bin-default";
pub const OUTPUT_BIN_SUPPORTED: &'static str = "output-bin-supported";
pub const ORIENTATION_REQUESTED_DEFAULT: &'static str = "orientation-requested-default";
pub const ORIENTATION_REQUESTED_SUPPORTED: &'static str = "orientation-requested-supported";
pub const MEDIA_DEFAULT: &'static str = "media-default";
pub const MEDIA_SUPPORTED: &'static str = "media-supported";
pub const PAGES_PER_MINUTE: &'static str = "pages-per-minute";
pub const COLOR_MODE_SUPPORTED: &'static str = "color-mode-supported";
pub const PRINT_COLOR_MODE_SUPPORTED: &'static str = "print-color-mode-supported";

enum_from_primitive! {
pub enum PrinterState {
    Idle = 3,
    Processing = 4,
    Stopped = 5,
}
}

enum_from_primitive! {
pub enum Orientation {
    Portrait = 3,
    Landscape = 4,
    ReverseLandscape = 5,
    ReversePortrait = 6,
}
}

enum_from_primitive! {
pub enum PrintQuality {
    Draft = 3,
    Normal = 4,
    High = 5,
}
}

enum_from_primitive! {
pub enum Finishings {
    None = 3,
    Staple = 4,
    Punch = 5,
    Cover = 6,
    Bind = 7,
    SaddleStitch = 8,
    EdgeStitch = 9,
}
}

enum_from_primitive! {
pub enum JobState {
    Pending = 3,
    PendingHeld = 4,
    Processing = 5,
    ProcessingStopped = 6,
    Canceled = 7,
    Aborted = 8,
    Completed = 9,
}
}
