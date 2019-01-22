# Main source of documentation

The main source of information is taken from the STM32CubeL4.
The low level implementation can be found under `Documentation/Driver/BSP/Components/mfxstml152`.

# Interface

There is 2 interface USART and I2C. The discovery board is only connected to the I2C by default.
There is also a MFX_WAKEUP line so we would need a GPIOPin.

I2C run at 100Khz. This value is taken from the stm32cubel4 example.

# Register description

## CTRL: Control register Offset: 0x80

offset bit:

|       7 |        6 | 5   4  | 3    2    1 |   0 |
|---------|----------|--------|-------------|-----|
| CAL_DIS | VREF_DIS | UNUSED | SHUNT_NB    | REQ |
|---------|----------|--------|-------------|-----|

  * CAL_DIS: Specifies if calibration is done before each Idd measurement
             0 ENABLED, 1 DISABLED
  * VREF_DIS: Specifies if Vref is automatically measured before each Idd measurement
              0 ENABLED, 1 DISABLED
  * SHUNT_NB: Specifies number of shunts used for measurement
  * REQ: Start a request.

## PRE_DELAY:

## SHUNT REGISTER:

There is 5 shunt register in other to configure the ohm for each one of them. Size is u16.

   * SHUNT0_MSB  0x82
   * SHUNT0_LSB  0x83
   * SHUNT1_MSB  0x84
   * SHUNT1_LSB  0x85
   * SHUNT2_MSB  0x86
   * SHUNT2_LSB  0x87
   * SHUNT3_MSB  0x88
   * SHUNT3_LSB  0x89
   * SHUNT4_MSB  0x8A
   * SHUNT4_LSB  0x8B

## SHUNT_ON_BOARD:

Size u8

## ADR_FW_VERSION Offset: 0x01

Size u16
