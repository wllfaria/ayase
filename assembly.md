# AYA Assembly

## Instruction Expansion
Aya assembly bracketed expression blocks accept some level of complex
instructions, allowing you to do math operations within the brackets, which
makes coding more convenient, but they must follow some rules.

1. If involving registers, only addition and subtraction can be performed.
2. You are allowed to do arbitrary math if there are only values or variables

Those instructions will be expanded, and no register will be changed, below are 
some examples of how the expansion will look like:

```asm
; NOTE: LitReg Instruction
mov r2, [r2 + r3]

; NOTE: expansion of left hand side expression has nothing to be done.

; expansion of right hand side expression
psh fp
mov fp, r2
add fp, r3

; performing original instruction
; NOTE: since we had no left side expansion, we can just move and restore.
mov r2, fp

; restoring original values
pop fp
```

```asm
mov 
```

```asm
; INFO: LitMem Instruction
mov &[r1 + r3], [r2 + r3]

; expansion of left hand side expression
psh acc
mov acc, r1
add acc, r3

; expansion of right hand side expression
psh fp
mov fp, r2
add fp, r3

; performing original instruction
; NOTE: since here we have a memory address instruction, we don't need to move
;       the value in order to ensure the correct register ends with the value
mov &[acc], fp

; restoring original values
pop fp
pop acc
```

## Sample Instructions Syntax

```asm
; Move instructions
mov r1,         $3000       ; mov literal into register                     (MovLitReg)
mov r1,         r2          ; mov register into register                    (MovRegReg)
mov &[$c0d3],   r3          ; mov register into memory                      (MovRegMem)
mov r1,         &[$3000]    ; mov memory into register                      (MovMemReg)
mov &[$3000],   $abcd       ; mov literal into memory                       (MovLitMem)
mov r1,         &[r2]       ; mov register pointer into register            (MovRegPtrReg)

; Math instructions
add r1,         r2          ; add register into register                    (AddRegReg)
add r1,         $0010       ; add literal into register                     (AddLitReg)
sub r1,         r2          ; sub register from register                    (SubRegReg)
sub r1,         $0010       ; sub literal from register                     (SubLitReg)
mul r1,         r2          ; multiply register with register               (MulRegReg)
mul r1,         $0010       ; multiply register with literal                (MulLitReg)
inc r1                      ; increment register                            (IncReg)
dec r1                      ; decrement register                            (DecReg)

; Binary instructions
lsh r1,         r2          ; left shift register with register             (LsfRegReg)
lsh r1,         $0010       ; left shift register with literal              (LsfLitReg)
rsh r1,         r2          ; right shift register with register            (RsfRegReg)
rsh r1,         $0010       ; right shift register with literal             (RsfLitReg)
and r1,         r2          ; and (&) register into register                (AndRegReg)
and r1,         $0010       ; and (&) literal into register                 (AndLitReg)
or  r1,         r2          ; or  (|) register into register                (OrRegReg)
or  r1,         $0010       ; or  (|) literal into register                 (OrLitReg)
xor r1,         r2          ; xor (^) register into register                (XorRegReg)
xor r1,         $0010       ; xor (^) literal into register                 (XorLitReg)
not r1                      ; not (~) register                              (Not)

; Memory instructions
psh r1                      ; push register into stack                      (PushReg)
psh $0010                   ; push literal into stack                       (PushLit)
pop r1                      ; pop from the stack into register              (Pop)
call &[$0100]               ; call subroutine on address                    (Call)
ret                         ; return from subroutine                        (Ret)

; Jump instructions
jeq &[$0000],   r2          ; jumps if register is equal to ret             (JeqReg)
jeq &[$0000],   $0000       ; jumps if literal is equal to ret              (JeqLit)
jgt &[$0000],   r2          ; jumps if register is greater than ret         (JgtReg)
jgt &[$0000],   $0000       ; jumps if literal is greater than ret          (JgtLit)
jne &[$0000],   r2          ; jumps if register is not equal to ret         (JneReg)
jne &[$0000],   $0000       ; jumps if literal is not equal to ret          (JneLit)
jge &[$0000],   r2          ; jumps if register is greater or equal to ret  (JgeReg)
jge &[$0000],   $0000       ; jumps if literal is greater or equal to ret   (JgeLit)
jle &[$0000],   r2          ; jumps if register is lesser or equal to ret   (JleReg)
jle &[$0000],   $0000       ; jumps if literal is lesser or equal to ret    (JleLit)
jlt &[$0000],   r2          ; jumps if register is lesser than ret          (JltReg)
jlt &[$0000],   $0000       ; jumps if literal is lesser than ret           (JltLit)
hlt                         ; halts the virtual machine                     (Halt)
```

## Module Import Syntax
```asm
; Module system syntax
import "./path.aya" ModuleName &[abcd] {
    variable1: !var,
    variable2: $0000,
    variable3: &[$0000],
    variable4: [OtherModule.variable],
}
```
