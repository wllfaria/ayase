# module resolution

when starting to 

```asm
; Move Operations
mov r1 $3000    ; mov literal into register (MovLitReg)
mov r1 r2       ; mov register into register (MovRegReg)
mov &c0d3 r3    ; mov register into memory (MovRegMem)
mov r1 &3000    ; mov memory into register (MovMemReg)
mov &3000 $abcd ; mov literal into memory (MovLitMem)
mov r1 &r2      ; mov register pointer into register (MovRegPtrReg)

; Complex move operations
mov &[r1 + $0010] r2     ; mov value on r2 into the address pointed by r2 + literal

; binary operations
add r1 r2       ; add register into register (AddRegReg)
add r1 $0010    ; add literal into register (AddLitReg)
sub r1 $0010    ; sub literal from register (SubLitReg)
sub $0010 r1    ; sub register from literal (SubRegLit)
sub r1 r2       ; sub register from register (SubRegReg)
inc r1          ; increment register (IncReg)
dec r1          ; decrement register (DecReg)
mul r1 $0010    ; multiply register with literal (MulLitReg)
mul r1 r2       ; multiply register with register (MulRegReg)

; bitwise operations
lsh r1 $0010    ; left shift register with literal (LsfLitReg)
lsh r1 r2       ; left shift register with register (LsfRegReg)
rsh r1 $0010    ; right shift register with literal (RsfLitReg)
rsh r1 r2       ; right shift register with register (RsfRegReg)
and r1 $0010    ; and (&) literal into register (AndLitReg)
and r1 r2       ; and (&) register into register (AndRegReg)
or  r1 $0010    ; or  (|) literal into register (OrLitReg)
or  r1 r2       ; or  (|) register into register (OrRegReg)
xor r1 $0010    ; xor (^) literal into register (OrRegReg)
xor r1 r2       ; xor (^) register into register (OrRegReg)
not r1          ; not (~) register (Not)

; stack operations (?)
psh $0010       ; push literal into stack (PushLit)
psh r1          ; push register into stack (PushReg)
psh &r1         ; push register pointer into stack (PushRegPtr)
pop             ; pop from the stack (Pop)
pop r1          ; pop from the stack into register (PopReg)
cal &0100       ; call subroutine on address (Call)
cal &r1         ; call subroutine from register pointer (CallRegPtr)
ret             ; return from subroutine

hlt             ; halts the virtual machine
```

```asm
module main

use "./some_module.aya" some_module $0303
use "./data_app.aya" data_app $ff33

mov r1, [$42 + !loc - ($05 * ($31 + !var) - $07)]
mov r1, [!var]
```
