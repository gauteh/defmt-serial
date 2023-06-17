#! /usr/bin/env bash
echo "flashing with openocd"
openocd -f interface/raspberrypi-swd.cfg -f target/rp2040.cfg -c "program $1 verify reset exit"


echo "Attaching defmt-print.."
cat /dev/serial0 | defmt-print -e $1