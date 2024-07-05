#![no_std]
#![no_main]


use arduino_hal::prelude::*;
use arduino_hal::spi;

use embedded_hal::digital::OutputPin;
use max7219::*;

use panic_halt as _;

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

    /*
     * For examples (and inspiration), head to
     *
     *     https://github.com/Rahix/avr-hal/tree/main/examples
     *
     * NOTE: Not all examples were ported to all boards!  There is a good chance though, that code
     * for a different board can be adapted for yours.  The Arduino Uno currently has the most
     * examples available.
     */

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
    display.set_intensity(0, 0x00).and_then(|_| {
        println!("Changed intensity to max\r");
        Ok(())
    }).or_else(|_| {
        println!("Failed change intensity\r");
        Err(())
    }).unwrap();
    // make sure to wake the display up
    display.power_off().and_then(|_| {
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

    let mut state = true;
    loop {
        println!("State: {}\r", state);
        display.test(0, state).unwrap();
        led.set_state(state.into()).unwrap();

        state = !state;
        arduino_hal::delay_ms(1000);
    }
}