#![no_std]
#![no_main]

use ch32v00x_hal as hal;
use ch32v_rt::entry;
use chrono::NaiveDateTime;
use hal::{i2c::*, pac::Peripherals, prelude::*};
mod rx8900;
use rx8900::Rx8900;
use fugit::HertzU32 as Hertz;
use embedded_hal::blocking::delay::DelayMs;
use hal::println;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

struct Ch32Delay {
    sysclock: Hertz,
}

impl Ch32Delay {
    fn new(sysclock: Hertz) -> Self {
        Self { sysclock }
    }
}

impl DelayMs<u32> for Ch32Delay
{
    fn delay_ms(&mut self, ms: u32) {
        let cycles = self.sysclock.raw() / 1000 * ms / 2;
        unsafe {
            qingke::riscv::asm::delay(cycles);
        }
    }
}

#[entry]
fn main() -> ! {
    #[cfg(feature = "sdi_print")]
    hal::debug::SDIPrint::enable();

    let pac = Peripherals::take().unwrap();

    let mut rcc = pac.RCC.constrain();
    let clocks = rcc.config.freeze();

    let gpioa = pac.GPIOA.split(&mut rcc);
    #[cfg(feature = "rtc")]
    let gpioc = pac.GPIOC.split(&mut rcc);
    let mut red = gpioa.pa1.into_push_pull_output();
    let mut green = gpioa.pa2.into_push_pull_output();
    #[cfg(feature = "rtc")]
    let sda = gpioc.pc1.into_alternate_open_drain();
    #[cfg(feature = "rtc")]
    let scl = gpioc.pc2.into_alternate_open_drain();
    #[cfg(feature = "rtc")]
    let i2c = I2c::i2c1(
        pac.I2C1,
        scl,
        sda,
        I2cConfig::fast_mode(),
        &mut rcc,
        &clocks,
    );

    let mut delay = Ch32Delay::new(clocks.sysclk());

    #[cfg(feature = "rtc")]
    let mut rx8900 = Rx8900::new(i2c);

    #[cfg(feature = "rtc")]
    match rx8900.voltage_low_flag() {
        Ok(true) => {
            #[cfg(feature = "sdi_print")]
            println!("Voltage low flag is set");
            delay.delay_ms(1000u32);
            rx8900.init().unwrap();
            rx8900.set_datetime(NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            )).unwrap();
        },
        #[cfg(feature = "sdi_print")]
        Ok(false) => {
            println!("Voltage low flag is not set");
        },
        #[cfg(feature = "sdi_print")]
        Err(e) => {
            println!("Error occured: {:?}", e);
        },
        _ => {}
    }

    #[cfg(feature = "rtc")]
    let mut counter: usize = 0;
    #[cfg(feature = "rtc")]
    let mut last_second: u8 = core::u8::MAX;

    loop {
        #[cfg(feature = "rtc")]
        {
            let second = rx8900.sec().unwrap();
            if second == last_second {
                delay.delay_ms(10u32);
                continue;
            }

            if counter < 20 {
                counter += 1;
            } else if counter < 30 {
                counter += 1;
                green.set_low();
                red.set_high();
                delay.delay_ms(500);
                green.set_high();
                red.set_low();
                delay.delay_ms(400);
                continue;
            } else {
                green.set_high();
                red.set_high();
                rx8900.set_datetime(NaiveDateTime::new(
                    chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                )).unwrap();
                continue;
            }

            last_second = second;

            let minutes = rx8900.min().unwrap();
            #[cfg(feature = "sdi_print")]
            println!("{:02}:{:02}", minutes, second);

            match minutes {
                55..=59 => {
                    // 待ち合わせ時間直前5分間は赤を点滅
                    green.set_low();
                    red.set_high();
                    delay.delay_ms(500);
                    red.set_low();
                    delay.delay_ms(400);
                },
                0..=4 => {
                    // 5分間の待ち合わせ時間内は緑を点灯
                    green.set_high();
                    red.set_low();
                    delay.delay_ms(900);
                },
                5..=9 => {
                    // 待ち合わせ時間が過ぎたら5分間緑を点滅
                    green.set_high();
                    red.set_low();
                    delay.delay_ms(500);
                    green.set_low();
                    delay.delay_ms(400);
                },
                _ => {
                    // それ以外の場合は赤を点灯
                    green.set_low();
                    red.set_high();
                    delay.delay_ms(900);
                }
            }
        }

        #[cfg(not(feature = "rtc"))]
        {
            green.set_high();
            red.set_high();
            delay.delay_ms(500);
            green.set_low();
            delay.delay_ms(400);
        }
    }
}
