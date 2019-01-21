# Main source of documentation

The main source of information is taken from the STM32CubeL4.
The low level implementation can be found under `Documentation/Driver/BSP/Components/mfxstml152`.

# Interface

There is 2 interface USART and I2C. The discovery board is only connected to the I2C by default.
There is also a MFX_WAKEUP line so we would need a GPIOPin.

# Register description

## CTRL: Control register

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
