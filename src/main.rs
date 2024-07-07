#![no_std]
#![no_main]
#![feature(generic_const_exprs)]

#[allow(unused_imports)]
use arduino_hal::prelude::*;
use arduino_hal::spi;

use embedded_hal::digital::OutputPin;
use max7219::*;

use panic_halt as _;

mod led_control;
use led_control::LedControl;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // set up serial interface for text output
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    macro_rules! println {
        ($($y:expr),+) => {
            ufmt::uwriteln!(&mut serial, $($y),+).ok()
        };
    }
    println!("Hello!\r");

    let mut led = pins.d2.into_output();
    led.set_low();

    let clk = pins.d13.into_output(); // sck
    let mosi = pins.d11.into_output(); // data
    let miso = pins.d12.into_pull_up_input();
    let cs = pins.d10.into_output(); // cs

    let (spi, cs_pin) = arduino_hal::Spi::new(
        dp.SPI,
        clk,
        mosi,
        miso,
        cs,
        spi::Settings {
            data_order: spi::DataOrder::MostSignificantFirst,
            clock: spi::SerialClockRate::OscfOver4,
            mode: embedded_hal::spi::MODE_0,
        },
    );

    let mut display = MAX7219::from_spi_cs(1, spi, cs_pin).unwrap();

    // set display intensity lower
    display.set_intensity(0, 0x0F).and_then(|_| {
        println!("Changed intensity to max\r");
        Ok(())
    }).or_else(|_| {
        println!("Failed change intensity\r");
        Err(())
    }).unwrap();
    // make sure to wake the display up
    display.power_on().and_then(|_| {
        println!("Powered on\r");
        Ok(())
    }).or_else(|_| {
        println!("Failed to power on\r");
        Err(())
    }).unwrap();
    // clear display
    display.clear_display(0).and_then(|_| {
        println!("Display cleared\r");
        Ok(())
    }).or_else(|_| {
        println!("Failed to clear display\r");
        Err(())
    }).unwrap();

    let mut lc: LedControl<1,_> = LedControl::new(display);

    let mut state = true;
    let addr = 0;
    let lc_ref = &mut lc;
    loop {
        row_demo(lc_ref, addr);
        counter_demo(lc_ref, addr);
        number_demo(lc_ref, addr);
        led.set_state(state.into()).unwrap();
        state = !state;
        arduino_hal::delay_ms(1000);
    }
}

#[allow(dead_code)]
fn row_demo<const N: usize, CONNECTOR>(lc_ref: &mut LedControl<N, CONNECTOR>, addr: usize)
where
    CONNECTOR: connectors::Connector,
    [(); 8 * N]:,
{
    for i in 0..4 {
        for shift in (0..=7).rev() {
            let value = 1 << shift;
            lc_ref.clear_display(addr).ok();
            lc_ref.set_row(addr, i, value).ok();
            arduino_hal::delay_ms(450);
        }
    }
}

#[allow(dead_code)]
fn counter_demo<const N: usize, CONNECTOR>(lc_ref: &mut LedControl<N, CONNECTOR>, addr: usize)
where
    CONNECTOR: connectors::Connector,
    [(); 8 * N]:,
{
    let step = 1; // divided by 10
    let demo_delay = 250;

    lc_ref.set_digit(addr, 0, 0, false).ok();
    lc_ref.set_digit(addr, 1, 0, false).ok();
    lc_ref.set_digit(addr, 2, 0, false).ok();
    lc_ref.set_digit(addr, 3, 0, false).ok();
    arduino_hal::delay_ms(demo_delay * 2);

    let mut int_part = 0usize;
    let mut dec_part = 0usize;
    while int_part != 1000 {
        let val_array: [usize; 3] = [
            (int_part/100*100-int_part/1000*1000)/100,
            (int_part/10*10-int_part/100*100)/10,
            int_part-int_part/10*10
        ];

        lc_ref.clear_display(addr).ok();
        let mut hide_zeros = true;
        if int_part == 0 {
            lc_ref.set_digit(addr, 2, 0, true).ok();
            lc_ref.set_digit(addr, 3, dec_part, false).ok();
        } else {
            for i in 0..=2 {
                match (val_array[i], hide_zeros) {
                    (0, true) => {}
                    (_, true) => {
                        hide_zeros = false;
                        lc_ref.set_digit(addr, i as u8, val_array[i], i == 2).ok();
                    },
                    (_, false) => {
                        lc_ref.set_digit(addr, i as u8, val_array[i], i == 2).ok();
                    }
                };
            }
            lc_ref.set_digit(addr, 3, dec_part, false).ok();
        }

        dec_part += step;
        if dec_part >= 10 {
            int_part += dec_part / 10;
            dec_part = dec_part % 10;
        }
        arduino_hal::delay_ms(demo_delay);
    }
}


#[allow(dead_code)]
fn number_demo<const N: usize, CONNECTOR>(lc_ref: &mut LedControl<N, CONNECTOR>, addr: usize)
where
    CONNECTOR: connectors::Connector,
    [(); 8 * N]:,
{
    lc_ref.set_digit(addr, 0, 0, false).ok();
    lc_ref.set_digit(addr, 1, 0, false).ok();
    lc_ref.set_digit(addr, 2, 0, false).ok();
    lc_ref.set_digit(addr, 3, 0, false).ok();
    arduino_hal::delay_ms(300);

    for digit in 0..=3 {
        for value in 1..=9 {
            lc_ref.set_digit(addr, 3-digit, value, false).ok();
            arduino_hal::delay_ms(300);
        }
    }
}
