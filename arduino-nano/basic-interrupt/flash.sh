#! /usr/bin/zsh

set -e

if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "usage: $0 <path-to-binary.elf>" >&2
    exit 1
fi

if [ "$#" -lt 1 ]; then
    echo "$0: Expecting a .elf file" >&2
    exit 1
fi

sudo cargo +nightly-2021-01-07 build -Z build-std=core --target avr-atmega328p.json --release
avrdude -patmega328p -carduino -D "-Uflash:w:$1:e" -P/dev/tty.usbserial-141440