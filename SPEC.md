# AYA SPEC

## Registers
ACC - Accumulator Register
Part of the calling convention, sub routines that return values should set this
register as the value, or address to the value

IP - Instruction Pointer
Will always point to the next instruction to be executed

R1-R4
Non volatile general purpose registers, these registers are guaranteed to keep
the data in them between function calls

R5-R8
Volatile general purpose registers, there are no guarantees that the values held
by these registers wont be changed between function calls

SP
Stack pointer, this register will always point to the next available address in
the stack

FP
Frame pointer, this register will always point to the base of the current stack
frame

## Calling Convention
R1 - First argument to sub routine
R2 - Second argument to sub routine
R3 - Third argument to sub routine
R4 - Fourth argument to sub routine
Acc - Will store the return value

## Instructions

## Memory Layout
| START  | END    | DESCRIPTION                                                    |
|--------|--------|----------------------------------------------------------------|
| 0x0000 | 0x3FFF | 16KiB Memory dedicated to hold the programs source code        |
| 0x4000 | 0x4FFF | 4KiB Memory dedicated to hold sprites, see [sprites](#sprites) |
| 0x5000 | 0x6FFF | 8KiB memory dedicated to drawing to the screen, partitioned    |
| TODO: Rest of the memory layout                                                  |
| 0xE000 | 0xFFFF | 8KiB stack memory                                              |


## Sprite Section
Sprites are represented in memory as as a 32 byte sequence, where each byte
stores 2 pixels. The VM has 2 palettes, and each palette has 8 colors, this
allow us to specify any of the colors, and which palette with 4 bits. The format
will use the 3 least significant bits as the color index, and the most 
significant bit as which the palette to use.

```
     +- Left pixel
     |
     |         +- Right pixel
     |         |
  vvvvvvv   vvvvvvv
+---------+---------+
| 1 0 1 1 | 0 1 0 1 |
+---------+---------+
  ^ ^^^^^
  |   |
  |   +- Color index
  |
  +- Palette index
```
