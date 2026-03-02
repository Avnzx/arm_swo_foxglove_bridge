[![Crates.io](https://img.shields.io/crates/v/arm_swo_foxglove_bridge.svg)](https://crates.io/crates/arm_swo_foxglove_bridge)
[![License](https://img.shields.io/crates/l/arm_swo_foxglove_bridge.svg)](https://github.com/Avnzx/arm_swo_foxglove_bridge#license)
[![Downloads](https://img.shields.io/crates/d/arm_swo_foxglove_bridge.svg)](https://crates.io/crates/arm_swo_foxglove_bridge)

# About

This tool splits ARM ITM values (with the TPIU formatter bypassed!) into
different, configurable, 'streams', that are sent to a compatible log sink
using the foxglove SDK.

# Using the tool
## Run OpenOCD

Example script for an STM32F4:

```tcl
source [find interface/stlink-dap.cfg]
source [find target/stm32f4x.cfg]


# Configure the TPIU for SWO output
stm32f4x.tpiu configure -protocol uart -traceclk 168000000 -pin-freq 2000000 -output :3344 -formatter off -port-width 1

# Enable the TPIU
stm32f4x.tpiu enable
itm ports on
# Initialize and reset the target to apply settings
init
stm32f4x.cpu arm semihosting enable
```

## Run a foxglove compatible sink

Run a foxglove-sdk compatible sink, such as foxglove or lichtblick.

## Run the tool

`arm_swo_foxglove_bridge /path/to/some/config/file.ron`



# ITM Sender Example 

[OWR 502](https://github.com/bluesatunsw/owr-502/blob/main/src/embedded_common/src/debug.rs)
