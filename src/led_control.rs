use max7219::*;

pub enum LedControlError {
    InvalidAddress,
    ValueError(&'static str),
    DataError(max7219::DataError)
}

pub type LedControlResult = Result<(), LedControlError>;

macro_rules! handle_err {
    ($y:expr) => {
        match $y {
            Ok(_) => Ok(()),
            Err(e) => Err(LedControlError::DataError(e))
        }
    };
}

pub struct LedControl<const N: usize, CONNECTOR>
where
    CONNECTOR: connectors::Connector,
    [(); 8 * N]:,
{
    display: MAX7219<CONNECTOR>,
    status: [u8; 8 * N],
}

#[allow(dead_code)]
impl<const N: usize, CONNECTOR> LedControl<N, CONNECTOR>
where
    CONNECTOR: connectors::Connector,
    [(); 8 * N]:,
{
    pub fn new(mut display: MAX7219<CONNECTOR>) -> Self
    {
        assert!(N > 0, "N must be greater than 0");
        assert!(N <= 8, "N must be lower or equal than 8");

        display.set_decode_mode(0, DecodeMode::NoDecode).ok();

        Self {
            display,
            status: [0u8; 8*N],
        }
    }

    pub fn destroy(self) -> MAX7219<CONNECTOR>
    {
        self.display
    }

    pub const fn get_device_count() -> usize
    {
        N
    }

    pub fn shutdown(&mut self, addr: usize, state: bool) -> LedControlResult
    {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }

        handle_err!(self.display.write_raw_byte(addr, Command::Power as u8, if state == true { 1 } else { 0 }))
    }

    pub fn set_scan_limit(&mut self, addr: usize, limit: u8) -> LedControlResult {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }
        if limit > 8 {
            return Err(LedControlError::ValueError("limit must be lower or equal than 8"));
        }

        handle_err!(self.display.write_raw_byte(addr, Command::ScanLimit as u8, limit))
    }

    pub fn set_intensity(&mut self, addr: usize, intensity: u8) -> LedControlResult {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }
        if intensity >= 16 {
            return Err(LedControlError::ValueError("intensity must be lower than 16"));
        }

        handle_err!(self.display.write_raw_byte(addr, Command::Intensity as u8, intensity))
    }

    pub fn clear_display(&mut self, addr: usize) -> LedControlResult {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }

        let offset = addr * 8;
        let opt_err_found = (0u8..8).find_map(|i| {
            self.status[offset+i as usize]=0;
            let res = self.display.write_raw_byte(addr, i+1, 0);

            match res {
                Ok(()) => None,
                Err(e) => Some(e)
            }
        });

        match opt_err_found {
            Some(err) => Err(LedControlError::DataError(err)),
            None => Ok(())
        }
    }

    pub fn set_led(&mut self, addr: usize, row: u8, col: u8, state: bool) -> LedControlResult
    {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }
        if row > 7 {
            return Err(LedControlError::ValueError("row must be lower than 8"));
        }
        if col > 7 {
            return Err(LedControlError::ValueError("col must be lower than 8"));
        }

        let offset = addr*8;
        let mut val: u8 = 0b10000000 >> col;

        let index = offset+row as usize;
        if state {
            self.status[index] = self.status[index] | val;
        } else {
            val = !val;
            self.status[index] = self.status[index] & val;
        }

        handle_err!(self.display.write_raw_byte(addr, row+1, self.status[index]))
    }

    pub fn set_row(&mut self, addr: usize, row: u8, value: u8) -> LedControlResult
    {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }
        if row > 7 {
            return Err(LedControlError::ValueError("row must be lower than 8"));
        }

        let opcode = row + 1;
        handle_err!(self.display.write_raw_byte(addr, opcode, value))
    }

    pub fn set_column(&mut self, addr: usize, col: u8, value: u8) -> LedControlResult
    {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }

        if col > 7 {
            return Err(LedControlError::ValueError("col must be lower than 8"));
        }

        let opt_err_found = (0u8..8).find_map(|row| {
            let mut val = value >> (7 - row);
            val = val & 0x01;

            let res = self.set_led(addr, row, col, val == 1);

            match res {
                Ok(()) => None,
                Err(e) => Some(e)
            }
        });

        match opt_err_found {
            None => Ok(()),
            Some(err) => Err(err)
        }
    }

    pub fn set_digit(&mut self, addr: usize, digit: u8, value: usize, dp: bool) -> LedControlResult
    {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }
        if digit > 7 {
            return Err(LedControlError::ValueError("digit must be lower than 8"));
        }

        let opcode = digit + 1;
        let mut v = NUMBERS[value];
        if dp {
            v |= 0b10000000;
        }

        let offset = addr*8;
        self.status[offset+digit as usize]=v;
        handle_err!(self.display.write_raw_byte(addr, opcode, v))
    }

    pub fn set_char(&mut self, addr: usize, digit: u8, value: char, dp: bool) -> LedControlResult
    {
        if addr >= N {
            return Err(LedControlError::InvalidAddress);
        }
        if digit > 7 {
            return Err(LedControlError::ValueError("digit must be lower than 8"));
        }

        let offset = addr*8;
        let mut index = value as usize;
        if index > 127 {
            index = 32;
        }
        let mut v = CHAR_TABLE[index];
        if dp {
            v |= 0b10000000;
        }

        self.status[offset+digit as usize] = v;
        handle_err!(self.display.write_raw_byte(addr, digit+1, v))
    }
}

const NUMBERS: [u8; 10] = [
    0b01111110,0b00110000,0b01101101,0b01111001,0b00110011,
    0b01011011,0b01011111,0b01110000,0b01111111,0b01111011
];

const CHAR_TABLE: [u8; 128] = [
    0b01111110,0b00110000,0b01101101,0b01111001,0b00110011,0b01011011,0b01011111,0b01110000,
    0b01111111,0b01111011,0b01110111,0b00011111,0b00001101,0b00111101,0b01001111,0b01000111,
    0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,
    0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,
    0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,
    0b00000000,0b00000000,0b00000000,0b00000000,0b10000000,0b00000001,0b10000000,0b00000000,
    0b01111110,0b00110000,0b01101101,0b01111001,0b00110011,0b01011011,0b01011111,0b01110000,
    0b01111111,0b01111011,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,
    0b00000000,0b01110111,0b00011111,0b00001101,0b00111101,0b01001111,0b01000111,0b00000000,
    0b00110111,0b00000000,0b00000000,0b00000000,0b00001110,0b00000000,0b00000000,0b00000000,
    0b01100111,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,
    0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00001000,
    0b00000000,0b01110111,0b00011111,0b00001101,0b00111101,0b01001111,0b01000111,0b00000000,
    0b00110111,0b00000000,0b00000000,0b00000000,0b00001110,0b00000000,0b00010101,0b00011101,
    0b01100111,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,
    0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000,0b00000000
];
