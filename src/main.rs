#![no_std]
#![no_main]

use riscv_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let RCC_APB2PCENR: *mut u32 = 0x4002_1018 as _;
    let GPIOC_CFGLR: *mut u32 = 0x4001_1000 as _;
    let GPIOC_OUTDR: *mut u32 = 0x4001_100C as _;

    unsafe {
        // Enable clocks to the GPIOC bank
        RCC_APB2PCENR.write_volatile(0b1_0000);
        // Set pin 1 to output
        GPIOC_CFGLR.write_volatile(0b0001_0000);

        loop {
            // Set pin 1 to high
            GPIOC_OUTDR.write_volatile(0b1_0);

            for _ in 0..1_000_000 {
                core::hint::black_box(()); // Do nothing, but keep the loop
            }

            // Set pin 1 to low
            GPIOC_OUTDR.write_volatile(0b0_0);

            for _ in 0..1_000_000 {
                core::hint::black_box(()); // Do nothing, but keep the loop
            }
        }
    }
}