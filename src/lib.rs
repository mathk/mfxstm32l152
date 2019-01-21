#![no_std]

extern crate embedded_hal as hal;

use core::mem;

use hal::blocking::i2c::{Write, WriteRead};
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

impl Register {
    pub fn addr(self) -> u8 {
        self as u8
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
    I2C: WriteRead<Error = E> + Write<Error = E>,
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

    pub fn config_shunt0(&mut self, data: u16, stabDelay: u8 ) -> Result<(), E> {
        self.config_shunt(Register::SHUNT0, data, Register::SH0_STABILIZATION, stabDelay)
    }

    pub fn config_shunt1(&mut self, data: u16, stabDelay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT1, data, Register::SH1_STABILIZATION, stabDelay)
    }

    pub fn config_shunt2(&mut self, data: u16, stabDelay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT2, data, Register::SH2_STABILIZATION, stabDelay)
    }

    pub fn config_shunt3(&mut self, data: u16, stabDelay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT3, data, Register::SH3_STABILIZATION, stabDelay)
    }

    pub fn config_shunt4(&mut self, data: u16, stabDelay: u8) -> Result<(), E> {
        self.config_shunt(Register::SHUNT4, data, Register::SH4_STABILIZATION, stabDelay)
    }

    fn config_shunt(&mut self, regShunt: Register, data: u16, regDelay: Register, stabDelay: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[regShunt.addr(), (data >> 8) as u8, (data & 0xFF) as u8])?;
        self.i2c.write(self.address, &[regDelay.addr(), stabDelay])
    }

    pub fn last_shunt_used(&mut self) -> Result<u8, E> {
        let mut buffer: [u8; 1] = unsafe { mem::uninitialized() };
        self.i2c.write_read(self.address, &[Register::SHUNT_USED.addr()], &mut buffer)?;
        Ok(buffer[0])
    }

    fn write_register(&mut self, reg: Register, data: u8) -> Result<(), E> {
        self.i2c.write(self.address, &[reg.addr(), data])
    }
}
