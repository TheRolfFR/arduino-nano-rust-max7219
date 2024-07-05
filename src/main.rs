#![no_std]
#![no_main]

#[allow(unused_imports)]
use arduino_hal::prelude::*;
use arduino_hal::spi;

use embedded_hal::digital::OutputPin;
use max7219::*;

use panic_halt as _;

const NUMBERS: [u8; 10] = [
    0b01111110,0b00110000,0b01101101,0b01111001,0b00110011,
    0b01011011,0b01011111,0b01110000,0b01111111,0b01111011
];


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

    let mut state = true;
    let addr = 0;
    loop {
        set_digit(&mut display, addr, 0, 0, false);
        set_digit(&mut display, addr, 1, 0, false);
        set_digit(&mut display, addr, 2, 0, false);
        set_digit(&mut display, addr, 3, 0, false);
        arduino_hal::delay_ms(450);

        for i in 0usize..=9999 {
            let mut dp_array: [bool; 4] = [false; 4];
            let val_array: [usize; 4] = [
                i/1000,
                (i/100*100-i/1000*1000)/100,
                (i/10*10-i/100*100)/10,
                i-i/10*10
            ];

            let mut val = i;
            let mut index = 0usize;
            while val > 10 {
                val = val / 10;
                index = index + 1;
            }
            dp_array[index] = true;

            println!("{} => {:?}, {:?}", i, val_array, dp_array);

            for i in 0usize..=3 {
                set_digit(&mut display, addr, i as u8, val_array[i], dp_array[i]);
            }


            arduino_hal::delay_ms(250);
        }

        led.set_state(state.into()).unwrap();
        state = !state;
        arduino_hal::delay_ms(1000);
    }
}

fn number_demo<CONNECTOR>(display: &mut MAX7219<CONNECTOR>, addr: usize)
where
    CONNECTOR: connectors::Connector,
{
    set_digit(display, addr, 0, 0, false);
    set_digit(display, addr, 1, 0, false);
    set_digit(display, addr, 2, 0, false);
    set_digit(display, addr, 3, 0, false);
    arduino_hal::delay_ms(300);

    for digit in 0..=3 {
        for value in 1..=9 {
            set_digit(display, addr, 3-digit, value, false);
            arduino_hal::delay_ms(300);
        }
    }
}

fn set_digit<CONNECTOR>(display: &mut MAX7219<CONNECTOR>, addr: usize, digit: u8, value: usize, dp: bool)
where
    CONNECTOR: connectors::Connector,
{
    display.set_decode_mode(0, DecodeMode::NoDecode).ok();

    let opcode = digit + 1;
    let mut v = NUMBERS[value];
    if dp {
        v |= 0b10000000;
    }
    display.write_raw_byte(addr, opcode, v).ok();
}