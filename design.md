# Instruction set design

16 bit instructions

## Immediate

Add immediate, sub immediate, shift left and right, and, or, xor

- 3 bits source, 3 bits destination register

- 3 bit opcode designator

- 3 bit function designator

- remaining 7 bits are immediate value

## LUI

- 3 bits destination register

- 3 bit opcode designator opcode

- 10 bits are used to put in the 10 top bits of destination, the rest zeroed

## Register to register

ADD, SUB,
AND, OR, XOR
SLT signed compare
SLTU unsigned compare

## Conditional stop

STEQ, STNE, STLT, STGE

## Load and store

16 bit load with constant offset

LOAD

16 bit store with constant offset
STORE
