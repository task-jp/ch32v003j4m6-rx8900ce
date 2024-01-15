#![no_std]
#![no_main]

use ch32v_rt::entry;
use ch32v00x_hal::prelude::*;
use ch32v00x_hal::pac::Peripherals;
use panic_halt as _;

#[entry]
fn main() -> ! {
    let pac = Peripherals::take().unwrap();

    let mut rcc = pac.RCC.constrain();
    let _clocks = rcc.config.freeze();

    let gpioc = pac.GPIOC.split(&mut rcc);
    let mut led = gpioc.pc1.into_push_pull_output();
    loop {
        led.toggle();

        for _ in 0..1_000_000 {
            core::hint::black_box(()); // Do nothing, but keep the loop
        }
    }
}
