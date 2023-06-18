#! /usr/bin/env bash
echo "flashing with openocd"
elf2uf2-rs -d $1

echo "Attaching defmt-print.."
cat /dev/ttyS0 | defmt-print -e $1