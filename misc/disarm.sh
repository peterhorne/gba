#!/usr/bin/env sh

echo $* | tr ' ' '\n' | while read hex; do printf \\x$hex; done > /tmp/disarm.bin
arm-none-eabi-objdump -D -b binary -m armv4t /tmp/disarm.bin | tail -n 1 | cut -f 3- | cut -d ';' -f 1
