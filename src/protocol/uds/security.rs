use defmt::{debug, info, warn};
use heapless::Vec;
use super::*;

// Security level definitions
const SECURITY_LEVEL_UNLOCKED: u8 = 0x01;

/// UDS Security Access implementation
pub struct SecurityAccess {
    /// Current security level
    security_level: u8,
    /// Whether the security access is unlocked
    unlocked: bool,
    /// Counter for failed unlock attempts
    failed_attempts: u8,
    /// Maximum allowed failed unlock attempts
    max_failed_attempts: u8,
    /// Seed value for last challenge
    last_seed: u32,
}

impl SecurityAccess {
    /// Create a new security access handler
    pub fn new() -> Self {
        Self {
            security_level: 0,
            unlocked: false,
            failed_attempts: 0,
            max_failed_attempts: 3,
            last_seed: 0,
        }
    }
    
    /// Initialize security access
    pub fn init(&mut self) {
        debug!("Initializing security access");
        self.security_level = 0;
        self.unlocked = false;
        self.failed_attempts = 0;
        self.last_seed = 0;
    }
    
    /// Handle security access service
    pub fn handle_security_access(&mut self, data: &[u8]) -> Vec<u8, 64> {
        if data.is_empty() {
            return self.create_negative_response(
                UDS_SID_SECURITY_ACCESS, 
                UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
            );
        }
        
        let subfunction = data[0];
        
        // Check if requesting seed (odd subfunction)
        if (subfunction & 0x01) == 0x01 {
            return self.handle_seed_request(subfunction);
        } 
        // Check if sending key (even subfunction)
        else if (subfunction & 0x01) == 0x00 {
            if data.len() < 5 {
                return self.create_negative_response(
                    UDS_SID_SECURITY_ACCESS, 
                    UDS_NRC_CONDITIONS_NOT_CORRECT
                );
            }
            
            // Extract key from data
            let key = u32::from_be_bytes([data[1], data[2], data[3], data[4]]);
            return self.handle_key_verification(subfunction, key);
        } 
        else {
            // Invalid subfunction
            return self.create_negative_response(
                UDS_SID_SECURITY_ACCESS, 
                UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
            );
        }
    }
    
    /// Handle seed request
    fn handle_seed_request(&mut self, subfunction: u8) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        // Verify the security level matches what we support
        if subfunction != SECURITY_LEVEL_UNLOCKED {
            return self.create_negative_response(
                UDS_SID_SECURITY_ACCESS, 
                UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
            );
        }
        
        // Check if already unlocked for this level
        if self.unlocked {
            // Already unlocked, return zero seed
            response.push(UDS_SID_SECURITY_ACCESS + UDS_RSP_POSITIVE);
            response.push(subfunction);
            response.push(0);
            response.push(0);
            response.push(0);
            response.push(0);
            return response;
        }
        
        // Check if we have exceeded maximum attempts
        if self.failed_attempts >= self.max_failed_attempts {
            return self.create_negative_response(
                UDS_SID_SECURITY_ACCESS, 
                UDS_NRC_EXCEEDED_NUMBER_OF_ATTEMPTS
            );
        }
        
        // Generate seed value (would be more complex in real implementation)
        self.last_seed = self.generate_seed();
        
        // Build positive response with seed
        response.push(UDS_SID_SECURITY_ACCESS + UDS_RSP_POSITIVE);
        response.push(subfunction);
        
        // Add seed bytes (big-endian)
        let seed_bytes = self.last_seed.to_be_bytes();
        response.push(seed_bytes[0]);
        response.push(seed_bytes[1]);
        response.push(seed_bytes[2]);
        response.push(seed_bytes[3]);
        
        response
    }
    
    /// Handle key verification
    fn handle_key_verification(&mut self, subfunction: u8, key: u32) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        // Verify the security level matches what we support
        if subfunction != SECURITY_LEVEL_UNLOCKED + 1 {
            return self.create_negative_response(
                UDS_SID_SECURITY_ACCESS, 
                UDS_NRC_SUB_FUNCTION_NOT_SUPPORTED
            );
        }
        
        // Check if already unlocked
        if self.unlocked {
            response.push(UDS_SID_SECURITY_ACCESS + UDS_RSP_POSITIVE);
            response.push(subfunction);
            return response;
        }
        
        // Check if we have exceeded maximum attempts
        if self.failed_attempts >= self.max_failed_attempts {
            return self.create_negative_response(
                UDS_SID_SECURITY_ACCESS, 
                UDS_NRC_EXCEEDED_NUMBER_OF_ATTEMPTS
            );
        }
        
        // Calculate expected key from seed
        let expected_key = self.calculate_key(self.last_seed);
        
        // Verify key
        if key == expected_key {
            // Successful unlock
            self.unlocked = true;
            self.security_level = SECURITY_LEVEL_UNLOCKED;
            self.failed_attempts = 0;
            
            info!("Security access unlocked");
            
            response.push(UDS_SID_SECURITY_ACCESS + UDS_RSP_POSITIVE);
            response.push(subfunction);
        } else {
            // Failed unlock attempt
            self.failed_attempts += 1;
            
            warn!("Invalid security key, attempt {}/{}", 
                 self.failed_attempts, self.max_failed_attempts);
            
            return self.create_negative_response(
                UDS_SID_SECURITY_ACCESS, 
                UDS_NRC_INVALID_KEY
            );
        }
        
        response
    }
    
    /// Generate seed value (simplified version)
    fn generate_seed(&self) -> u32 {
        // In a real implementation, this would be more complex
        // and use some device-specific values
        
        // Get some pseudo-random value
        let timer_value = 0x12345678; // Replace with actual timer value
        
        // XOR with some constant
        timer_value ^ 0xA5A5A5A5
    }
    
    /// Calculate expected key from seed (simplified version)
    fn calculate_key(&self, seed: u32) -> u32 {
        // In a real implementation, this would be a more complex algorithm
        // XOR with a constant and bit rotation (simple example)
        let mut key = seed ^ 0x5A5A5A5A;
        
        // Rotate right by 3 bits
        key = (key >> 3) | (key << 29);
        
        key
    }
    
    /// Check if security access is unlocked
    pub fn is_unlocked(&self) -> bool {
        self.unlocked
    }
    
    /// Create a negative response
    fn create_negative_response(&self, sid: u8, nrc: u8) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        response.push(UDS_SID_NEGATIVE_RESPONSE);
        response.push(sid);
        response.push(nrc);
        
        response
    }
}