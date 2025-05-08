# mb2-touch: MicroBit v2 Touch Pad driver
Bart Massey 2023 (version 0.2.0)

This code demos capacitive touch sensing for the gold robot
"logo pad" and other designated "touch pads" on the MicroBit
v2. It works by driving the pad low for 5ms to drain any
on-board capacitance, then measuring the time for the pad to
charge through the 10MΩ resistor on the board until it reads
high. On my board, it takes less than 50µs to charge with no
finger, about 2.5ms to charge with a finger firmly planted.

This is currently a lib crate with some examples.

# License

This work is licensed under the "MIT License". Please see the file
`LICENSE.txt` in this distribution for license terms.
