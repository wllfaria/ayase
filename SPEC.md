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
| START  | END    | DESCRIPTION                                                |
|--------|--------|------------------------------------------------------------|
| 0x0000 | 0x1FFF |  8KiB Memory dedicated to hold [tiles](#tiles-section)     |
| 0x2000 | 0x227F |  640B Memory dedicated to sprite drawing                   |
| 0x2280 | 0x627F | 16KiB Memory dedicated to program source code              |
| 0x6280 | 0x6423 |  420B Memory for background tilemap drawing                |
| 0x6424 | 0x65C7 |  420B Memory for interface tilemap drawing                 |
| 0x65C8 | 0x65D7 |   16B Memory as interrupt table                            |
| 0x65D7 | 0x65D8 |    1B Memory as input mapping                              |
| TODO: Rest of the memory layout                                              |
| 0xE000 | 0xFFFF | 8KiB stack memory                                          |

## Graphics

### Tiles Section
Graphics are rendered as tiles rather than managing individual pixels. Each tile
is made of 8x8 squares, tiles does not encode any special information, they are
only composed by index into palette colors for each of its pixels, ranging from
0 to 15, meaning tiles are 4bpp (4 bits per pixel), which lets us encode 2 
pixels into a byte, meaning each tile uses 32 total bytes, which allow you to 
store 256 total tiles in the total 8KiB available for tiles, see 
[memory layout](#memory-layout).


#### In-memory tile representation
```txt
     +- Left pixel
     |
     |         +- Right pixel
     |         |
  vvvvvvv   vvvvvvv
+---------+---------+
| 1 0 1 1 | 0 1 0 1 |
+---------+---------+
  ^^^^^^^
     |
     +- Color index
```

### Palette
Aya has a single palette made up with 16 different colors, where 15 of those 
are solid colors, and one is transparent. Transparent color is 0th index of the
color palette.

### Sprite Section
Sprites are individual movable entities that are based on tiles, but allow for
better control over how it is rendered. up to 40 sprites can be drawn to the 
screen at any point. Each sprite is composed by a 16 byte structure that goes
as follows:

| BYTE    | DESCRIPTION                                                        |
|---------|--------------------------------------------------------------------|
|  00     | Sprite's X position onscreen                                       |
|  01     | Sprite's Y position onscreen                                       |
|  02     | Tile index                                                         |
|  03     | Animation frame                                                    |
|  04     | Sprite attribute flags, see [Sprite flags](#sprite-flags)          |
|  05-15  | 11 bits to be used as the programmer desires                       |

#### Sprite Flags
Sprite flags is a bitmasked byte that defines how a sprite should be drawn, each
bit has a special meaning that goes as follows:

| Byte 0 | Byte 1 | Byte 2 - Byte 7 |
|--------|--------|-----------------|
| x flip | y flip | TODO            |
