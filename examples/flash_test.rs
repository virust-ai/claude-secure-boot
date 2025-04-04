#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use defmt::{info, error};
use defmt_rtt as _;

use gridania_telematic_bootloader::bootloader::flash::Flash;
use gridania_telematic_bootloader::bootloader::verification::FirmwareVerification;
use gridania_telematic_bootloader::drivers::clock::Clock;

/// Flash memory testing example
#[entry]
fn main() -> ! {
    info!("Flash Test Example");
    
    // Initialize clock
    let mut clock = Clock::new();
    clock.init();
    
    // Create flash controller
    let mut flash = Flash::new();
    flash.init();
    
    // Create verification tool
    let verification = FirmwareVerification::new();
    
    // Test flash operations
    test_flash_operations(&mut flash, &verification);
    
    // Loop forever
    loop {
        cortex_m::asm::nop();
    }
}

/// Test flash operations
fn test_flash_operations(flash: &mut Flash, verification: &FirmwareVerification) {
    info!("Starting flash operations test");
    
    // Define test area (adjust based on memory map)
    let test_addr = 0x00010000;  // Example address - should be in application area
    let test_size = 1024;        // 1KB
    
    // Test data to write
    let test_data = [0x55; 128]; // 128 bytes of 0x55
    
    // Step 1: Erase flash sector
    info!("Erasing flash at 0x{:08X}, size: {}", test_addr, test_size);
    match flash.erase(test_addr, test_size) {
        Ok(_) => info!("Flash erase successful"),
        Err(e) => {
            error!("Flash erase failed: {:?}", e);
            return;
        }
    }
    
    // Step 2: Write test data
    info!("Writing test data to flash");
    match flash.write(test_addr, &test_data) {
        Ok(_) => info!("Flash write successful"),
        Err(e) => {
            error!("Flash write failed: {:?}", e);
            return;
        }
    }
    
    // Step 3: Verify data
    info!("Verifying written data");
    let mut verify_passed = true;
    
    // Directly read from memory address to verify
    for i in 0..test_data.len() {
        let value = unsafe { 
            core::ptr::read_volatile((test_addr + i as u32) as *const u8) 
        };
        
        if value != test_data[i] {
            error!("Verification failed at offset {}: expected 0x{:02X}, got 0x{:02X}", 
                   i, test_data[i], value);
            verify_passed = false;
            break;
        }
    }
    
    if verify_passed {
        info!("Flash verification successful");
    } else {
        error!("Flash verification failed");
    }
    
    // Step 4: Test checksum functionality
    info!("Testing checksum calculation");
    let checksum = verification.calculate_checksum(test_addr, test_data.len() as u32);
    info!("Calculated checksum: 0x{:08X}", checksum);
}