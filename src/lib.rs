#![no_std]

extern crate cast;
extern crate embedded_hal as hal;
extern crate i2c_hal_tools;


use i2c_hal_tools::noincrement::NoIncrementI2c;
use i2c_hal_tools::autoincrement::AutoIncrementI2c;
use i2c_hal_tools::{SerialRead, SerialWrite};
use hal::blocking::delay::{DelayUs};
use hal::digital::OutputPin;
use i2c_hal_tools::Register as R;

use core::fmt;


pub struct Ampere {
    value: u32,
    exponent: u8,
}

impl Ampere {

    pub fn new(value: u32, exponent: u8) -> Self {
        Self {
            value,
            exponent,
        }
    }
}

impl fmt::Display for Ampere {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (unit, div) = if self.exponent < 3 {
            ("m", 10u32.pow(3 - self.exponent as u32))
        } else if self.exponent < 6 {
            ("u", 10u32.pow(6 - self.exponent as u32))
        } else {
            ("n", 10u32.pow(9 - self.exponent as u32))
        };
        write!(f, "{}{}A", self.value * div, unit)
    }
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum RoRegister {
    // Chip ID
    ADR_ID = 0x00,

    // Firmware version
    ADR_FW_VERSION = 0x01,

    // Read error code 0 ok, 1 timeout, 2 no value
    ERROR_MSG = 0x04,

    // Read the number of shunt being used in the last idd read
    IDD_SHUNT_USED = 0x1A,

    // Value in 24 bits MSB MID LSB
    IDD_VALUE = 0x14,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum DelayUnit {
    TIME_5_MS = 0x00,
    TIME_20_MS = 0x80,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum Register {


    // System control register
    SYS_CTRL = 0x40,

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
    // Vdd Min value u16
    IDD_VDD_MIN = 0x8E, // 0x8F is the MSB

    // Shunt stabilization in millisecond
    IDD_SH0_STABILIZATION = 0x90,
    IDD_SH1_STABILIZATION = 0x91,
    IDD_SH2_STABILIZATION = 0x92,
    IDD_SH3_STABILIZATION = 0x93,
    IDD_SH4_STABILIZATION = 0x94,

    IDD_NBR_OF_MEAS = 0x96,

    // Delay between each measurment
    IDD_MEAS_DELTA_DELAY = 0x97,

    // Shunt on board
    IDD_SHUNTS_ON_BOARD = 0x98,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
pub enum NbShunt {
    SHUNT_NB_1 = 0x01,
    SHUNT_NB_2 = 0x02,
    SHUNT_NB_3 = 0x03,
    SHUNT_NB_4 = 0x04,
    SHUNT_NB_5 = 0x05,
}

#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
enum SysCtrl {
    SWRST = 0x80,
    STANDBY = 0x40,
    ALTERNATE_GPIO_EN = 0x08, //* by the way if IDD and TS are enabled they take automatically the AF pins*/
    IDD_EN = 0x04,
    TS_EN = 0x02,
    GPIO_EN = 0x01,
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
    I2C: SerialRead<AutoIncrementI2c, RoRegister, Error = E> + SerialRead<NoIncrementI2c, Register, Error = E> + SerialWrite<NoIncrementI2c, Register, Error = E>,
    GPIO: OutputPin,
    Delay: DelayUs<u32>,
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
        let mut mode = self.i2c.read_u8(self.address, Register::SYS_CTRL)?;
        mode |= SysCtrl::IDD_EN as u8;
        self.i2c.write_u8(self.address, Register::SYS_CTRL, mode)?;
        Ok(())
    }

    pub fn set_idd_ctrl(&mut self, calibration_disabled: bool, vref_disabled: bool, nb_shunt: NbShunt) -> Result<(), E> {
        let cal = if calibration_disabled {
            0x80
        } else {
            0x00
        };

        let vref = if vref_disabled {
            0x40
        } else {
            0x00
        };
        let nb_shunt = nb_shunt as u8;
        let value = (nb_shunt << 1) | cal | vref;
        self.i2c.write_u8(self.address, Register::IDD_CTRL, value)?;
        self.i2c.write_u8(self.address, Register::IDD_SHUNTS_ON_BOARD, nb_shunt)
    }

    pub fn set_idd_nb_measurment(&mut self, nb: u8) -> Result<(), E> {
        self.i2c.write_u8(self.address, Register::IDD_NBR_OF_MEAS, nb)
    }

    pub fn set_idd_shunt0(&mut self, data: u16, stab_delay: u8 ) -> Result<(), E> {
        self.set_idd_shunt(Register::IDD_SHUNT0, data, Register::IDD_SH0_STABILIZATION, stab_delay)
    }

    pub fn set_idd_shunt1(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.set_idd_shunt(Register::IDD_SHUNT1, data, Register::IDD_SH1_STABILIZATION, stab_delay)
    }

    pub fn set_idd_shunt2(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.set_idd_shunt(Register::IDD_SHUNT2, data, Register::IDD_SH2_STABILIZATION, stab_delay)
    }

    pub fn set_idd_shunt3(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.set_idd_shunt(Register::IDD_SHUNT3, data, Register::IDD_SH3_STABILIZATION, stab_delay)
    }

    pub fn set_idd_shunt4(&mut self, data: u16, stab_delay: u8) -> Result<(), E> {
        self.set_idd_shunt(Register::IDD_SHUNT4, data, Register::IDD_SH4_STABILIZATION, stab_delay)
    }

    fn set_idd_shunt(&mut self, reg_shunt: Register, data: u16, reg_delay: Register, stab_delay: u8) -> Result<(), E> {
        self.i2c.write_be_u16(self.address, reg_shunt, data)?;
        self.i2c.write_u8(self.address, reg_delay, stab_delay)
    }

    pub fn set_idd_gain(&mut self, value: u16) -> Result<(), E> {
        self.i2c.write_be_u16(self.address, Register::IDD_GAIN, value)
    }

    pub fn set_idd_pre_delay(&mut self, unit: DelayUnit, value: u8) -> Result<(), E>{
        let value = self.cap_delay_value(value);
        let unit = unit as u8;
        self.i2c.write_u8(self.address, Register::IDD_PRE_DELAY, unit & value)
    }

    pub fn set_idd_meas_delta_delay(&mut self, unit: DelayUnit, value: u8) -> Result<(), E> {
        let value = self.cap_delay_value(value);
        let unit = unit as u8;
        self.i2c.write_u8(self.address, Register::IDD_MEAS_DELTA_DELAY, unit & value)
    }

    pub fn set_idd_vdd_min(&mut self, value: u16) -> Result<(), E> {
        self.i2c.write_be_u16(self.address, Register::IDD_VDD_MIN, value)
    }

    pub fn idd_start(&mut self) -> Result<(), E> {
        let mut mode = self.i2c.read_u8(self.address, Register::IDD_CTRL)?;
        mode |= 1;
        self.i2c.write_u8(self.address, Register::IDD_CTRL, mode)
    }

    pub fn idd_get_value(&mut self) -> Result<Ampere, E> {
        // TODO: Fix delay, maybe use IT.
        self.delay.delay_us(500_000);
        self.delay.delay_us(500_000);
        self.delay.delay_us(500_000);
        self.delay.delay_us(500_000);
        let value = self.i2c.read_be_u24(self.address, RoRegister::IDD_VALUE)?;
        Ok(Ampere::new(value, 8))
    }

    pub fn idd_ctrl(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, Register::IDD_CTRL)
    }

    pub fn idd_last_shunt_used(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, RoRegister::IDD_SHUNT_USED)
    }

    pub fn idd_shunts_on_board(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, Register::IDD_SHUNTS_ON_BOARD)
    }

    pub fn error_code(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, RoRegister::ERROR_MSG)
    }

    pub fn chip_id(&mut self) -> Result<u8, E> {
        self.i2c.read_u8(self.address, RoRegister::ADR_ID)
    }

    pub fn firmware_version(&mut self) -> Result<u16, E> {
        self.i2c.read_be_u16(self.address, RoRegister::ADR_FW_VERSION)
    }

    fn cap_delay_value(&mut self, value: u8) -> u8 {
        if value > 0x80 {
            0x7F
        } else {
            value
        }
    }
}
