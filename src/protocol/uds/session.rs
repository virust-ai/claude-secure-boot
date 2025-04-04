use defmt::{debug, info, warn};
use heapless::Vec;
use super::*;
use super::services::UdsServices;
use super::security::SecurityAccess;
use super::transfer::TransferManager;
use crate::bootloader::timeout::TimeoutReset;

/// UDS Session management
pub struct UdsSession {
    /// Current session type
    current_session: u8,
    /// UDS services handler
    services: UdsServices,
    /// Security access handler
    security: SecurityAccess,
    /// Transfer manager for download operations
    transfer: TransferManager,
    /// Timeout reset handler reference
    timeout_reset: Option<*mut TimeoutReset>,
}

impl UdsSession {
    /// Create a new UDS session manager
    pub fn new() -> Self {
        Self {
            current_session: UDS_SESSION_DEFAULT,
            services: UdsServices::new(),
            security: SecurityAccess::new(),
            transfer: TransferManager::new(),
            timeout_reset: None,
        }
    }
    
    /// Initialize the UDS session
    pub fn init(&mut self) {
        info!("Initializing UDS session");
        self.current_session = UDS_SESSION_DEFAULT;
        self.services.init();
        self.security.init();
        self.transfer.init();
    }
    
    /// Register timeout reset handler
    pub fn register_timeout_reset(&mut self, timeout_reset: &mut TimeoutReset) {
        self.timeout_reset = Some(timeout_reset);
    }
    
    /// Process incoming UDS message
    pub fn process_message(&mut self, data: &[u8]) -> Vec<u8, 64> {
        if data.is_empty() {
            return Vec::new();
        }
        
        // Extract service ID from first byte
        let sid = data[0];
        
        // Notify timeout reset that a message was received if in programming session
        if self.current_session == UDS_SESSION_PROGRAMMING {
            if let Some(timeout_reset) = self.timeout_reset {
                // Safety: We know this pointer is valid
                unsafe {
                    if sid == UDS_SID_DIAGNOSTIC_SESSION_CONTROL {
                        (*timeout_reset).set_flashing_init();
                    } else if sid == UDS_SID_REQUEST_DOWNLOAD {
                        (*timeout_reset).set_flashing_started();
                    }
                }
            }
        }
        
        // Process the UDS service
        match sid {
            UDS_SID_DIAGNOSTIC_SESSION_CONTROL => {
                self.handle_session_control(&data[1..])
            },
            UDS_SID_ECU_RESET => {
                self.services.handle_ecu_reset(&data[1..])
            },
            UDS_SID_SECURITY_ACCESS => {
                self.security.handle_security_access(&data[1..])
            },
            UDS_SID_TESTER_PRESENT => {
                self.handle_tester_present(&data[1..])
            },
            UDS_SID_REQUEST_DOWNLOAD => {
                // Only allowed in programming session with security access
                if self.current_session != UDS_SESSION_PROGRAMMING {
                    self.create_negative_response(sid, UDS_NRC_CONDITIONS_NOT_CORRECT)
                } else if !self.security.is_unlocked() {
                    self.create_negative_response(sid, UDS_NRC_SECURITY_ACCESS_DENIED)
                } else {
                    self.transfer.handle_request_download(&data[1..])
                }
            },
            UDS_SID_TRANSFER_DATA => {
                // Only allowed in programming session with security access
                if self.current_session != UDS_SESSION_PROGRAMMING {
                    self.create_negative_response(sid, UDS_NRC_CONDITIONS_NOT_CORRECT)
                } else if !self.security.is_unlocked() {
                    self.create_negative_response(sid, UDS_NRC_SECURITY_ACCESS_DENIED)
                } else {
                    self.transfer.handle_transfer_data(&data[1..])
                }
            },
            UDS_SID_REQUEST_TRANSFER_EXIT => {
                // Only allowed in programming session with security access
                if self.current_session != UDS_SESSION_PROGRAMMING {
                    self.create_negative_response(sid, UDS_NRC_CONDITIONS_NOT_CORRECT)
                } else if !self.security.is_unlocked() {
                    self.create_negative_response(sid, UDS_NRC_SECURITY_ACCESS_DENIED)
                } else {
                    self.transfer.handle_transfer_exit(&data[1..])
                }
            },
            _ => {
                // Unsupported service
                warn!("Unsupported UDS service: 0x{:02X}", sid);
                self.create_negative_response(sid, UDS_NRC_SERVICE_NOT_SUPPORTED)
            }
        }
    }
    
    /// Handle diagnostic session control
    fn handle_session_control(&mut self, data: &[u8]) -> Vec<u8, 64> {
        if data.is_empty() {
            return self.create_negative_response(
                UDS_SID_DIAGNOSTIC_SESSION_CONTROL, 
                UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
            );
        }
        
        let session_type = data[0];
        let mut response = Vec::new();
        
        match session_type {
            UDS_SESSION_DEFAULT | UDS_SESSION_PROGRAMMING | UDS_SESSION_EXTENDED => {
                // Set the new session type
                self.current_session = session_type;
                info!("UDS Session changed to 0x{:02X}", session_type);
                
                // Create positive response
                response.push(UDS_SID_DIAGNOSTIC_SESSION_CONTROL + UDS_RSP_POSITIVE);
                response.push(session_type);
                
                // If entering programming session, notify timeout reset
                if session_type == UDS_SESSION_PROGRAMMING {
                    if let Some(timeout_reset) = self.timeout_reset {
                        // Safety: We know this pointer is valid
                        unsafe {
                            (*timeout_reset).set_flashing_init();
                        }
                    }
                }
            },
            _ => {
                // Unsupported session type
                warn!("Unsupported session type: 0x{:02X}", session_type);
                return self.create_negative_response(
                    UDS_SID_DIAGNOSTIC_SESSION_CONTROL, 
                    UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
                );
            }
        }
        
        response
    }
    
    /// Handle tester present message
    fn handle_tester_present(&self, data: &[u8]) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        // Validate subfunction
        if data.is_empty() {
            return self.create_negative_response(
                UDS_SID_TESTER_PRESENT, 
                UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
            );
        }
        
        let subfunction = data[0];
        
        // Only zero suppression bit (0x80) is allowed to be set
        if (subfunction & 0x7F) != 0x00 {
            return self.create_negative_response(
                UDS_SID_TESTER_PRESENT, 
                UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
            );
        }
        
        // Check if response is suppressed
        if (subfunction & 0x80) == 0 {
            // Create positive response
            response.push(UDS_SID_TESTER_PRESENT + UDS_RSP_POSITIVE);
            response.push(0x00); // Subfunction echo
        }
        
        response
    }
    
    /// Create a negative response
    fn create_negative_response(&self, sid: u8, nrc: u8) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        response.push(UDS_SID_NEGATIVE_RESPONSE);
        response.push(sid);
        response.push(nrc);
        
        response
    }
    
    /// Get current session type
    pub fn get_session_type(&self) -> u8 {
        self.current_session
    }
}