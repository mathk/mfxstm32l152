#![no_std]

extern crate cast;
extern crate embedded_hal as hal;
extern crate i2c_hal_tools;


use i2c_hal_tools::autoincrement::AutoIncrementI2c;
use i2c_hal_tools::{SerialRead, SerialWrite};
use hal::blocking::delay::{DelayUs};
use hal::digital::OutputPin;
use i2c_hal_tools::Register as R;


#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum RoRegister {
    // Chip ID
    ADR_ID = 0x00,

    // Firmware version
    ADR_FW_VERSION = 0x01,

    // Read the number of shunt being used in the last idd read
    IDD_SHUNT_USED = 0x1A,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {
    // Idd control register (R/W)
    IDD_CTRL = 0x80,

    IDD_PRE_DELAY = 0x81,
    // Shunt resistor configuration
    // Lets do one incremental write
    IDD_SHUNT0 = 0x82, // MSB 0x83
    IDD_SHUNT1 = 0x84, // MSB 0x85,
    IDD_SHUNT2 = 0x86, // MSB 0x87,
    IDD_SHUNT3 = 0x88, // MSB 0x89,
    IDD_SHUNT4 = 0x8A, // MSB 0x8B,

    // Ampli gain
    IDD_GAIN = 0x8C, // 0x8B is the LSB

    // Shunt stabilization in millisecond
    IDD_SH0_STABILIZATION = 0x90,
    IDD_SH1_STABILIZATION = 0x91,
    IDD_SH2_STABILIZATION = 0x92,
    IDD_SH3_STABILIZATION = 0x93,
    IDD_SH4_STABILIZATION = 0x94,

    // Shunt on board
    IDD_SHUNTS_ON_BOARD = 0x98,
}

impl R for Register {
    fn addr(&self) -> u8 {
        *self as u8
    }
}


impl R for RoRegister {
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
    I2C: SerialRead<AutoIncrementI2c, RoRegister, Error = E> + SerialRead<AutoIncrementI2c, Register, Error = E> + SerialWrite<AutoIncrementI2c, Register, Error = E>,
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
        self.config_shunt(Register::IDD_SHUNT0, data, Register::IDD_SH0_STABILIZATION, stab_delay)
    }

    pub fn config_shunt1(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::IDD_SHUNT1, data, Register::IDD_SH1_STABILIZATION, stab_delay)
    }

    pub fn config_shunt2(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::IDD_SHUNT2, data, Register::IDD_SH2_STABILIZATION, stab_delay)
    }

    pub fn config_shunt3(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::IDD_SHUNT3, data, Register::IDD_SH3_STABILIZATION, stab_delay)
    }

    pub fn config_shunt4(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.config_shunt(Register::IDD_SHUNT4, data, Register::IDD_SH4_STABILIZATION, stab_delay)
    }

    fn config_shunt(&mut self, reg_shunt: Register, data: u16, reg_delay: Register, stab_delay: u8) -> Result<(), E> {
        self.i2c.write_be_u16(self.address, reg_shunt, data)?;
        self.i2c.write_u8(self.address, reg_delay, stab_delay)
    }

    pub fn last_shunt_used(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, RoRegister::IDD_SHUNT_USED)
    }

   pub fn shunts_on_board(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, Register::IDD_SHUNTS_ON_BOARD)
    }

    pub fn chip_id(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, RoRegister::ADR_ID)
    }

    pub fn firmware_version(&mut self) -> Result<u16, E> {
        self.i2c.read_be_u16(self.address, RoRegister::ADR_FW_VERSION)
    }
}
