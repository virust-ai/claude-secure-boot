#![no_std]
#![no_main]

// Import dependencies
use panic_halt as _;
use cortex_m_rt::entry;
use defmt::info;
use defmt_rtt as _;

// Import our modules
use gridania_telematic_bootloader::bootloader::core::BootLoader;
use gridania_telematic_bootloader::drivers::clock::Clock;
use gridania_telematic_bootloader::drivers::gpio::Gpio;
use gridania_telematic_bootloader::drivers::power::Power;
use gridania_telematic_bootloader::drivers::watchdog::Watchdog;

#[entry]
fn main() -> ! {
    info!("Gridania Telematic ECU Bootloader");
    info!("Build Date: {}", env!("CARGO_PKG_VERSION"));
    
    // Initialize core hardware components
    let mut clock = Clock::new();
    clock.init();
    
    let mut power = Power::new();
    power.init();
    
    let mut gpio = Gpio::new();
    gpio.init();
    
    let mut watchdog = Watchdog::new();
    watchdog.init();
    
    // Check HMI power state
    check_hmi_power(&gpio);
    
    // Initialize and run the bootloader
    let mut bootloader = BootLoader::new();
    bootloader.init();
    
    // Enable CPU interrupts
    unsafe { cortex_m::interrupt::enable() };
    
    // Main loop - continue processing bootloader tasks
    loop {
        // Run the bootloader task (handle communication, flashing, etc.)
        bootloader.task();
        
        // Service the watchdog
        watchdog.service();
    }
}

/// Check HMI power state and configure CAN transceiver accordingly
fn check_hmi_power(gpio: &Gpio) {
    let hmi_power_status = gpio.read_port_b() & (1 << 10);
    
    if hmi_power_status != 0 {
        info!("HMI is on, don't touch CAN transceiver");
    } else {
        info!("HMI is off, turning off CAN transceiver");
        gpio.set_port_c(5);
    }
}
