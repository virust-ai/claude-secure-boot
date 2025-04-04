pub mod services;
pub mod session;
pub mod security;
pub mod transfer;

// UDS Service IDs
pub const UDS_SID_DIAGNOSTIC_SESSION_CONTROL: u8 = 0x10;
pub const UDS_SID_ECU_RESET: u8 = 0x11;
pub const UDS_SID_SECURITY_ACCESS: u8 = 0x27;
pub const UDS_SID_COMMUNICATION_CONTROL: u8 = 0x28;
pub const UDS_SID_TESTER_PRESENT: u8 = 0x3E;
pub const UDS_SID_REQUEST_DOWNLOAD: u8 = 0x34;
pub const UDS_SID_TRANSFER_DATA: u8 = 0x36;
pub const UDS_SID_REQUEST_TRANSFER_EXIT: u8 = 0x37;
pub const UDS_SID_NEGATIVE_RESPONSE: u8 = 0x7F;

// UDS Response Codes
pub const UDS_RSP_POSITIVE: u8 = 0x40;  // Added to SID for positive response

// UDS Negative Response Codes
pub const UDS_NRC_GENERAL_REJECT: u8 = 0x10;
pub const UDS_NRC_SERVICE_NOT_SUPPORTED: u8 = 0x11;
pub const UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED: u8 = 0x12;
pub const UDS_NRC_CONDITIONS_NOT_CORRECT: u8 = 0x22;
pub const UDS_NRC_REQUEST_SEQUENCE_ERROR: u8 = 0x24;
pub const UDS_NRC_REQUEST_OUT_OF_RANGE: u8 = 0x31;
pub const UDS_NRC_SECURITY_ACCESS_DENIED: u8 = 0x33;
pub const UDS_NRC_INVALID_KEY: u8 = 0x35;
pub const UDS_NRC_EXCEEDED_NUMBER_OF_ATTEMPTS: u8 = 0x36;
pub const UDS_NRC_TRANSFER_DATA_SUSPENDED: u8 = 0x71;
pub const UDS_NRC_GENERAL_PROGRAMMING_FAILURE: u8 = 0x72;
pub const UDS_NRC_WRONG_BLOCK_SEQUENCE_COUNTER: u8 = 0x73;
pub const UDS_NRC_RESPONSE_PENDING: u8 = 0x78;

// Session Types
pub const UDS_SESSION_DEFAULT: u8 = 0x01;
pub const UDS_SESSION_PROGRAMMING: u8 = 0x02;
pub const UDS_SESSION_EXTENDED: u8 = 0x03;

// Reset Types
pub const UDS_RESET_HARD: u8 = 0x01;
pub const UDS_RESET_KEY_OFF_ON: u8 = 0x02;
pub const UDS_RESET_SOFT: u8 = 0x03;
pub const UDS_RESET_ENABLE_RAPID_POWER_SHUTDOWN: u8 = 0x04;
pub const UDS_RESET_DISABLE_RAPID_POWER_SHUTDOWN: u8 = 0x05;