use defmt::{debug, info, warn};
use heapless::Vec;
use super::*;
use crate::bootloader::flash::Flash;

// Transfer data constants
const MAX_MEMORY_SIZE: u32 = 0x100000; // 1MB max size
const MAX_BLOCK_SIZE: usize = 1024;    // 1KB block size

/// UDS Transfer data manager
pub struct TransferManager {
    /// Flash controller reference
    flash: Option<*mut Flash>,
    /// Current download address
    download_address: u32,
    /// Download size remaining
    download_size: u32,
    /// Block sequence counter
    block_counter: u8,
    /// Transfer in progress flag
    transfer_active: bool,
}

impl TransferManager {
    /// Create a new transfer manager
    pub fn new() -> Self {
        Self {
            flash: None,
            download_address: 0,
            download_size: 0,
            block_counter: 0,
            transfer_active: false,
        }
    }
    
    /// Initialize the transfer manager
    pub fn init(&mut self) {
        debug!("Initializing UDS transfer manager");
        self.download_address = 0;
        self.download_size = 0;
        self.block_counter = 0;
        self.transfer_active = false;
    }
    
    /// Register flash controller
    pub fn register_flash(&mut self, flash: &mut Flash) {
        self.flash = Some(flash);
    }
    
    /// Handle download request
    pub fn handle_request_download(&mut self, data: &[u8]) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        // Validate data length (at least 3 bytes for format, addr and size)
        if data.len() < 3 {
            return self.create_negative_response(
                UDS_SID_REQUEST_DOWNLOAD, 
                UDS_NRC_INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT
            );
        }
        
        // Extract address and size format
        let format = data[0];
        let addr_len = (format >> 4) & 0x0F;
        let size_len = format & 0x0F;
        
        // Validate length
        if data.len() < (1 + addr_len as usize + size_len as usize) {
            return self.create_negative_response(
                UDS_SID_REQUEST_DOWNLOAD, 
                UDS_NRC_INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT
            );
        }
        
        // Extract memory address
        let mut address: u32 = 0;
        for i in 0..addr_len as usize {
            address = (address << 8) | (data[1 + i] as u32);
        }
        
        // Extract memory size
        let mut size: u32 = 0;
        for i in 0..size_len as usize {
            size = (size << 8) | (data[1 + addr_len as usize + i] as u32);
        }
        
        // Validate address and size
        if !self.validate_memory_range(address, size) {
            return self.create_negative_response(
                UDS_SID_REQUEST_DOWNLOAD, 
                UDS_NRC_REQUEST_OUT_OF_RANGE
            );
        }
        
        // If valid, store download info and prepare for transfer
        self.download_address = address;
        self.download_size = size;
        self.block_counter = 0;
        self.transfer_active = true;
        
        info!("Download request: addr=0x{:08X}, size={}", address, size);
        
        // Erase flash memory if needed
        if let Some(flash) = self.flash {
            // Safety: We know this pointer is valid
            unsafe {
                match (*flash).erase(address, size) {
                    Ok(_) => {
                        debug!("Flash erase successful");
                    },
                    Err(_) => {
                        warn!("Flash erase failed");
                        return self.create_negative_response(
                            UDS_SID_REQUEST_DOWNLOAD, 
                            UDS_NRC_GENERAL_PROGRAMMING_FAILURE
                        );
                    }
                }
            }
        }
        
        // Create positive response
        response.push(UDS_SID_REQUEST_DOWNLOAD + UDS_RSP_POSITIVE);
        
        // Add max block size (single byte length format followed by 2 bytes size)
        response.push(0x10); // Length of max block size parameter (1 byte)
        response.push((MAX_BLOCK_SIZE >> 8) as u8);
        response.push(MAX_BLOCK_SIZE as u8);
        
        response
    }
    
    /// Handle transfer data
    pub fn handle_transfer_data(&mut self, data: &[u8]) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        // Check if transfer is active
        if !self.transfer_active {
            return self.create_negative_response(
                UDS_SID_TRANSFER_DATA, 
                UDS_NRC_REQUEST_SEQUENCE_ERROR
            );
        }
        
        // Validate data length (at least 1 byte for block counter)
        if data.is_empty() {
            return self.create_negative_response(
                UDS_SID_TRANSFER_DATA, 
                UDS_NRC_INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT
            );
        }
        
        // Extract block counter and verify sequence
        let block_counter = data[0];
        if block_counter != (self.block_counter + 1) {
            warn!("Block sequence error: expected={}, received={}", 
                 self.block_counter + 1, block_counter);
            return self.create_negative_response(
                UDS_SID_TRANSFER_DATA, 
                UDS_NRC_WRONG_BLOCK_SEQUENCE_COUNTER
            );
        }
        
        // Extract data to program
        let program_data = &data[1..];
        
        // Check if data size exceeds remaining size
        if program_data.len() as u32 > self.download_size {
            return self.create_negative_response(
                UDS_SID_TRANSFER_DATA, 
                UDS_NRC_REQUEST_OUT_OF_RANGE
            );
        }
        
        // Program data to flash
        if let Some(flash) = self.flash {
            // Safety: We know this pointer is valid
            unsafe {
                match (*flash).write(self.download_address, program_data) {
                    Ok(_) => {
                        debug!("Flash write successful");
                    },
                    Err(_) => {
                        warn!("Flash write failed");
                        return self.create_negative_response(
                            UDS_SID_TRANSFER_DATA, 
                            UDS_NRC_GENERAL_PROGRAMMING_FAILURE
                        );
                    }
                }
            }
        }
        
        // Update state
        self.download_address += program_data.len() as u32;
        self.download_size -= program_data.len() as u32;
        self.block_counter = block_counter;
        
        // Create positive response
        response.push(UDS_SID_TRANSFER_DATA + UDS_RSP_POSITIVE);
        response.push(block_counter);
        
        response
    }
    
    /// Handle transfer exit
    pub fn handle_transfer_exit(&mut self, _data: &[u8]) -> Vec<u8, 64> {
        let mut response = Vec::new();
        
        // Check if transfer is active
        if !self.transfer_active {
            return self.create_negative_response(
                UDS_SID_REQUEST_TRANSFER_EXIT, 
                UDS_NRC_REQUEST_SEQUENCE_ERROR
            );
        }
        
        // Finalize flash operations
        if let Some(flash) = self.flash {
            // Safety: We know this pointer is valid
            unsafe {
                match (*flash).finalize() {
                    Ok(_) => {
                        debug!("Flash finalization successful");
                    },
                    Err(_) => {
                        warn!("Flash finalization failed");
                        return self.create_negative_response(
                            UDS_SID_REQUEST_TRANSFER_EXIT, 
                            UDS_NRC_GENERAL_PROGRAMMING_FAILURE
                        );
                    }
                }
                
                // Write firmware checksum
                match (*flash).write_checksum() {
                    Ok(_) => {
                        debug!("Checksum write successful");
                    },
                    Err(_) => {
                        warn!("Checksum write failed");
                        return self.create_negative_response(
                            UDS_SID_REQUEST_TRANSFER_EXIT, 
                            UDS_NRC_GENERAL_PROGRAMMING_FAILURE
                        );
                    }
                }
            }
        }
        
        // Reset transfer state
        self.transfer_active = false;
        
        // Create positive response
        response.push(UDS_SID_REQUEST_TRANSFER_EXIT + UDS_RSP_POSITIVE);
        
        response
    }
    
    /// Validate memory address and size range
    fn validate_memory_range(&self, address: u32, size: u32) -> bool {
        // Check for overflow
        if size > MAX_MEMORY_SIZE {
            return false;
        }
        
        // Check if end address would overflow
        if address.checked_add(size).is_none() {
            return false;
        }
        
        // Check that the range is within the allowed flash area
        let app_start = 0x00008000; // Example value - adjust based on actual memory map
        let flash_end = 0x00080000; // Example value - adjust based on actual flash size
        
        address >= app_start && (address + size) <= flash_end
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

// Additional UDS NRC for transfer functions
const UDS_NRC_INCORRECT_MESSAGE_LENGTH_OR_INVALID_FORMAT: u8 = 0x13;