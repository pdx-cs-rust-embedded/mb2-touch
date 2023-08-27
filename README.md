# mb2-touch: MicroBit v2 Touch Logo driver
Bart Massey 2023

This code demos capacitive touch sensing for the gold robot
"logo pad" on the MicroBit v2. It works by driving the pad
low for 5ms to drain any on-board capacitance, then
measuring the time for the pad to charge through the 10MΩ
resistor on the board until it reads high. On my board, it
takes less than 50µs to charge with no finger, about 2.5ms
to charge with a finger firmly planted.
