```asm
section .data
message:
    .ascii "hello world"

section .text
global _start



module: symbolMap

%module "file"

%import module "file" {
    message: .ascii "hello world"
}

labels:

something:
    push 10
    call module.private
    

_start:
    mov rax, 0
    call 0x0000
```
