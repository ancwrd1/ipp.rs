//!
//! IPP operations
//!
pub const PRINT_JOB: u16 = 0x0002;
pub const PRINT_URI: u16 = 0x0003;
pub const VALIDATE_JOB: u16 = 0x0004;
pub const CREATE_JOB: u16 = 0x0005;
pub const SEND_DOCUMENT: u16 = 0x0006;
pub const SEND_URI: u16 = 0x0007;
pub const CANCEL_JOB: u16 = 0x0008;
pub const GET_JOB_ATTRIBUTES: u16 = 0x0009;
pub const GET_JOBS: u16 = 0x000A;
pub const GET_PRINTER_ATTRIBUTES: u16 = 0x000B;
pub const HOLD_JOB: u16 = 0x000C;
pub const RELEASE_JOB: u16 = 0x000D;
pub const RESTART_JOB: u16 = 0x000E;
pub const PAUSE_PRINTER: u16 = 0x0010;
pub const RESUME_PRINTER: u16 = 0x0011;
pub const PURGE_JOBS: u16 = 0x0012;

pub fn is_operation(value: u16) -> bool {
    value <= 0x12
}
