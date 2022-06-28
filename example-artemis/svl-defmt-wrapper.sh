#! /usr/bin/env bash

set -ep

TARGET=${1}
BIN="${TARGET}.bin"

BOOTLOADER="/home/gauteh/dev/ambiq-rs/tools/svl/svl.py"

echo "size:"
arm-none-eabi-size "${TARGET}"

echo "objcopy.."
arm-none-eabi-objcopy -S -O binary "${TARGET}" "${BIN}"

echo "flashing /dev/ttyUSB0.."
python3 "${BOOTLOADER}" -f "${BIN}" /dev/ttyUSB0 -v


echo "Attaching defmt-print.."

socat /dev/ttyUSB0,raw,echo=0 STDOUT | defmt-print -e $1
