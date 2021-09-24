[bits 64]

section .data
msg: db "Hello, World!", 10

global _start
section .text
_start:
  mov rax, 5                ; syscall number for WRITE
  mov rdi, 1                ; standard output
  mov rsi, msg              ; addr of string
  mov rdx, 14               ; size of string
  int 0x80
  jmp _start
