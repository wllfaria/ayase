# ROM Spec

## Header
Composed of 128 bytes, this header indicates where each section of the game 
should be loaded to memory, the header format is:

| OFFSET | SIZE     | DESCRIPTION                                              |
|--------|----------|----------------------------------------------------------|
| 0x0000 |  4 bytes | Magic file identifier to validate the ROM                |
| 0x0004 |  1 byte  | ROM version                                              |
| 0x0005 | 63 bytes | Game title, as a null terminated string                  |
| 0x0044 |  4 bytes | Code section offset                                      |
| 0x0048 |  4 bytes | Code section size                                        |
| 0x004c |  4 bytes | Sprite section offset                                    |
| 0x0050 |  4 bytes | Sprite section size                                      |

## Code Section
Contains the bytecode for the game, this will match the size specified in the
header, and should also respect the maximum size of 16KiB.

## Sprite Section
Packed version of the sprites to be used in the game, this will be bit packed
to conform with the sprite specification of the VM. This section size will match
the size specified on the header, being able to grow up to 4KiB.
