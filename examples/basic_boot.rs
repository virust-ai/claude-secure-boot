#![no_std]
#![no_main]

use panic_halt as _;
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;

use gridania_telematic_bootloader::bootloader::core::BootLoader;
use gridania_telematic_bootloader::drivers::clock::Clock;
use gridania_telematic_bootloader::drivers::gpio::Gpio;
use gridania_telematic_bootloader::drivers::power::Power;
use gridania_telematic_bootloader::drivers::watchdog::Watchdog;

/// Basic bootloader example showing initialization and core functionality
#[entry]
fn main() -> ! {
    info!("Basic Bootloader Example");
    
    // Initialize core hardware components
    let mut clock = Clock::new();
    clock.init();
    
    let mut power = Power::new();
    power.init();
    
    let mut gpio = Gpio::new();
    gpio.init();
    
    let mut watchdog = Watchdog::new();
    watchdog.init();
    
    // Initialize bootloader
    let mut bootloader = BootLoader::new();
    bootloader.init();
    
    // Enable CPU interrupts
    unsafe { cortex_m::interrupt::enable() };
    
    info!("Bootloader initialized");
    info!("Checking application...");
    
    // Check if valid application exists
    if bootloader.verify_application() {
        info!("Valid application found, starting...");
        bootloader.start_application();
    } else {
        info!("No valid application found, staying in bootloader");
    }
    
    // Main loop if no valid application
    loop {
        // Process bootloader tasks
        bootloader.task();
        
        // Service the watchdog
        watchdog.service();
    }
}