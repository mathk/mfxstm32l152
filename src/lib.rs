#![no_std]

extern crate cast;
extern crate embedded_hal as hal;
extern crate i2c_hal_tools;


use i2c_hal_tools::autoincrement::AutoIncrementI2c;
use i2c_hal_tools::{SerialRead, SerialWrite};
use hal::blocking::delay::{DelayUs};
use hal::digital::OutputPin;

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
    // Idd control register (R/W)
    CTRL = 0x80,
    // Read the number of shunt being used in the last idd read
    SHUNT_USED = 0x1A,

    // Shunt on board
    SHUNTS_ON_BOARD = 0x98,

    // Chip ID
    ADR_ID = 0x00,

    ADR_FW_VERSION = 0x01,

    // Ampli gain
    GAIN = 0x8C, // 0x8B is the LSB

    // Shunt resistor configuration
    // Lets do one incremental write
    SHUNT0 = 0x82, // MSB 0x83
    SHUNT1 = 0x84, // MSB 0x85,
    SHUNT2 = 0x86, // MSB 0x87,
    SHUNT3 = 0x88, // MSB 0x89,
    SHUNT4 = 0x8A, // MSB 0x8B,

    // Shunt stabilization in millisecond
    SH0_STABILIZATION = 0x90,
    SH1_STABILIZATION = 0x91,
    SH2_STABILIZATION = 0x92,
    SH3_STABILIZATION = 0x93,
    SH4_STABILIZATION = 0x94,
}

impl i2c_hal_tools::Register for Register {
    fn addr(&self) -> u8 {
        *self as u8
    }
}

pub struct MFX<I2C, GPIO, Delay> {
    i2c: I2C,
    wakup: GPIO,
    delay: Delay,
    address: u8,
}


impl<I2C, GPIO, Delay, E> MFX<I2C, GPIO, Delay>
where
    I2C: SerialRead<AutoIncrementI2c, Register, Error = E> + SerialWrite<AutoIncrementI2c, Register, Error = E>,
    GPIO: OutputPin,
    Delay: DelayUs<u8>,
{
    pub fn new(i2c: I2C, wakup: GPIO, delay: Delay, address: u8) -> Result<Self, E> {
        let mut mfx = Self {
            i2c,
            wakup,
            delay,
            address,
        };
        mfx.wakup()?;
        Ok(mfx)
    }

    fn wakup(&mut self) -> Result<(), E> {
        self.wakup.set_high();
        self.delay.delay_us(10);
        self.wakup.set_low();
        Ok(())
    }

    pub fn config_shunt0(&mut self, data: u16, stab_delay: u8 ) -> Result<(), E> {
        self.config_shunt(Register::SHUNT0, data, Register::SH0_STABILIZATION, stab_delay)
    }

    pub fn config_shunt1(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT1, data, Register::SH1_STABILIZATION, stab_delay)
    }

    pub fn config_shunt2(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT2, data, Register::SH2_STABILIZATION, stab_delay)
    }

    pub fn config_shunt3(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT3, data, Register::SH3_STABILIZATION, stab_delay)
    }

    pub fn config_shunt4(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT4, data, Register::SH4_STABILIZATION, stab_delay)
    }

    fn config_shunt(&mut self, reg_shunt: Register, data: u16, reg_delay: Register, stab_delay: u8) -> Result<(), E> {
        self.i2c.write_be_u16(self.address, reg_shunt, data)?;
        self.i2c.write_u8(self.address, reg_delay, stab_delay)
    }

    pub fn last_shunt_used(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, Register::SHUNT_USED)
    }

   pub fn shunts_on_board(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, Register::SHUNTS_ON_BOARD)
    }

    pub fn chip_id(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, Register::ADR_ID)
    }

    pub fn firmware_version(&mut self) -> Result<u16, E> {
        self.i2c.read_be_u16(self.address, Register::ADR_FW_VERSION)
    }
}
