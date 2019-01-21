#[no_std]

extern crate embedded_hal as hal;

use core::mem;

use hal::blocking::i2c::{Write, WriteRead};
use hal::blocking::delay::{DelayUs};
use hal::digital::OutputPin;


pub enum Register {
    // Idd control register (R/W)
    CTRL = 0x80,
    SHUNT_USED = 0x1A,

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

    pub fn last_shunt_used(&mut self) -> Result<u8, E> {
        let mut buffer: [u8; 1] = unsafe { mem::uninitialized() };
        self.i2c.write_read(self.address, &[Register::SHUNT_USED.addr()], &mut buffer)?;
        Ok(buffer[0])
    }

    fn write_register(&mut self, reg: Register, data: u8) -> Result<(), E> {
        self.i2c.write(
            self.address,
            &[reg.addr(), data],
        )
    }
}
