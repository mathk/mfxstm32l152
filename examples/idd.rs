//! Test the serial interface
//!
#![deny(unsafe_code)]
// #![deny(warnings)]
#![no_main]
#![no_std]

extern crate cortex_m;
#[macro_use(entry, exception)]
extern crate cortex_m_rt as rt;
// #[macro_use(block)]
// extern crate nb;
extern crate panic_semihosting;

extern crate stm32l4xx_hal as hal;
extern crate mfxstm32l152 as mfx;
// #[macro_use(block)]
// extern crate nb;

use cortex_m::asm;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::delay::Delay;
use crate::hal::i2c::I2c;
use crate::rt::ExceptionFrame;
use mfx::{MFX, DelayUnit, NbShunt};

use core::fmt::{self, Write};

static DISCOVERY_IDD_AMPLI_GAIN : u16 =  4967;   // value is gain * 100
// On rev B and A
// static DISCOVERY_IDD_AMPLI_GAIN : u16 =  4990;     /*!< value is gain * 100 */


struct Wrapper<'a> {
    buf: &'a mut [u8],
    offset: usize,
}

impl<'a> Wrapper<'a> {
    fn new(buf: &'a mut [u8]) -> Self {
        Wrapper {
            buf: buf,
            offset: 0,
        }
    }
}

impl<'a> fmt::Write for Wrapper<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();

        // Skip over already-copied data
        let remainder = &mut self.buf[self.offset..];
        // Check if there is space remaining (return error instead of panicking)
        if remainder.len() < bytes.len() { return Err(core::fmt::Error); }
        // Make the two slices the same length
        let remainder = &mut remainder[..bytes.len()];
        // Copy
        remainder.copy_from_slice(bytes);

        // Update offset to avoid overwriting
        self.offset += bytes.len();

        Ok(())
    }
}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let p = hal::stm32::Peripherals::take().unwrap();

    let mut flash = p.FLASH.constrain();
    let mut rcc = p.RCC.constrain();
    let mut gpiod = p.GPIOD.split(&mut rcc.ahb2);
    let mut gpiob = p.GPIOB.split(&mut rcc.ahb2);
    let mut gpioa = p.GPIOA.split(&mut rcc.ahb2);

    // clock configuration using the default settings (all clocks run at 8 MHz)
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    // TRY this alternate clock configuration (clocks run at nearly the maximum frequency)
    // let clocks = rcc.cfgr.sysclk(64.mhz()).pclk1(32.mhz()).freeze(&mut flash.acr);

    // VCOM gpio
    let tx = gpiod.pd5.into_af7(&mut gpiod.moder, &mut gpiod.afrl);
    let rx = gpiod.pd6.into_af7(&mut gpiod.moder, &mut gpiod.afrl);

    // MFX I2c
    let mut scl = gpiob.pb10.into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    scl.internal_pull_up(&mut gpiob.pupdr, true);
    let scl = scl.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

    let mut sda = gpiob.pb11.into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper);
    sda.internal_pull_up(&mut gpiob.pupdr, true);
    let sda = sda.into_af4(&mut gpiob.moder, &mut gpiob.afrh);

    // MFX Wakeup pin
    let wakup = gpioa.pa4.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper);

    let i2c = I2c::i2c2(p.I2C2, (scl, sda), 100.khz(), clocks, &mut rcc.apb1r1);
    let timer = Delay::new(cp.SYST, clocks);
    let mut mfx = MFX::new(i2c, wakup, timer, 0x84).unwrap();


    let serial = Serial::usart2(p.USART2, (tx, rx), 115_200.bps(), clocks, &mut rcc.apb1r1);
    let (mut tx, mut _rx) = serial.split();

    let fw = mfx.firmware_version().unwrap();
    let id = mfx.chip_id().unwrap();
    let mut buf = [0 as u8; 40];
    write!(Wrapper::new(&mut buf), "Firmware ID: {}, Chip ID: {}\n\r", fw, id).unwrap();

    tx.write_str(core::str::from_utf8(&buf).unwrap()).ok();

    // /**
    //  * @brief  Shunt values on discovery in milli ohms
    //  */
    // #define DISCOVERY_IDD_SHUNT0_VALUE                  ((uint16_t) 1000)     /*!< value in milliohm */
    // #define DISCOVERY_IDD_SHUNT1_VALUE                  ((uint16_t) 24)       /*!< value in ohm */
    // #define DISCOVERY_IDD_SHUNT2_VALUE                  ((uint16_t) 620)      /*!< value in ohm */
    // #define DISCOVERY_IDD_SHUNT4_VALUE                  ((uint16_t) 10000)    /*!< value in ohm */

    // /**
    //  * @brief  Shunt stabilization delay on discovery in milli ohms
    //  */
    // #define DISCOVERY_IDD_SHUNT0_STABDELAY              ((uint8_t) 149)       /*!< value in millisec */
    // #define DISCOVERY_IDD_SHUNT1_STABDELAY              ((uint8_t) 149)       /*!< value in millisec */
    // #define DISCOVERY_IDD_SHUNT2_STABDELAY              ((uint8_t) 149)       /*!< value in millisec */
    // #define DISCOVERY_IDD_SHUNT4_STABDELAY              ((uint8_t) 255)       /*!< value in millisec */


     // /**
     //   * @brief  Shunt values on discovery in milli ohms
     //   */
     // #define DISCOVERY_IDD_SHUNT0_VALUE                  ((uint16_t) 1000)     /*!< value in milliohm */
     // #define DISCOVERY_IDD_SHUNT1_VALUE                  ((uint16_t) 24)       /*!< value in ohm */
     // #define DISCOVERY_IDD_SHUNT2_VALUE                  ((uint16_t) 620)      /*!< value in ohm */
     // #define DISCOVERY_IDD_SHUNT4_VALUE                  ((uint16_t) 10000)    /*!< value in ohm */
     //
     // /**
     //   * @brief  Shunt stabilization delay on discovery in milli ohms
     //   */
     // #define DISCOVERY_IDD_SHUNT0_STABDELAY              ((uint8_t) 149)       /*!< value in millisec */
     // #define DISCOVERY_IDD_SHUNT1_STABDELAY              ((uint8_t) 149)       /*!< value in millisec */
     // #define DISCOVERY_IDD_SHUNT2_STABDELAY              ((uint8_t) 149)       /*!< value in millisec */
     // #define DISCOVERY_IDD_SHUNT4_STABDELAY              ((uint8_t) 255)       /*!< value in millisec */
     //
     // /**
     //   * @brief  IDD Ampli Gain on discovery
     //   */
     // #if defined(USE_STM32L476G_DISCO_REVC)
     // #define DISCOVERY_IDD_AMPLI_GAIN                    ((uint16_t) 4967)     /*!< value is gain * 100 */
     // #else
     // #define DISCOVERY_IDD_AMPLI_GAIN                    ((uint16_t) 4990)     /*!< value is gain * 100 */
     // #endif
     //
     // /**
     //   * @brief  IDD Vdd Min on discovery
     //   */
     // #define DISCOVERY_IDD_VDD_MIN                       ((uint16_t) 2000)     /*!< value in millivolt */

    mfx.set_idd_ctrl(false, false, NbShunt::SHUNT_NB_4).unwrap();
    mfx.set_idd_gain(DISCOVERY_IDD_AMPLI_GAIN).unwrap();
    mfx.set_idd_vdd_min(2000).unwrap(); // In milivolt
    mfx.set_idd_pre_delay(DelayUnit::TIME_20_MS, 0xF).unwrap(); // Max delay
    mfx.set_idd_shunt0(1000, 149).unwrap();
    mfx.set_idd_shunt1(24, 149).unwrap();
    mfx.set_idd_shunt2(620, 149).unwrap();
    mfx.set_idd_shunt3(0, 0).unwrap();
    mfx.set_idd_shunt4(10000, 255).unwrap();
    mfx.set_idd_nb_measurment(10).unwrap();
    mfx.set_idd_meas_delta_delay(DelayUnit::TIME_5_MS, 10).unwrap();

    mfx.idd_start().unwrap();

    let idd = mfx.idd_get_value().unwrap();
    let error = mfx.error_code().unwrap();

    let mut buf = [0 as u8; 40];
    write!(Wrapper::new(&mut buf), "\n\rIDD: {} nA, Error: {}\n\r\0", idd, error).unwrap();

    tx.write_str(core::str::from_utf8(&buf).unwrap()).ok();
    // when using virtual com port for recieve can causes a framing error
    // On the stm32l476 discovery it is working fine at 115200 baud


    // if all goes well you should reach this breakpoint
    asm::bkpt();

    loop {}
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("{:#?}", ef);
}
