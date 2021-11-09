[bits 64]

section .data
msg: db "Hello, World!", 10

global _start
section .text
_start:
  mov rax, 4                ; syscall number for WRITE
  mov rdi, 1                ; standard output
  mov rsi, msg              ; addr of string
  mov rdx, 14               ; size of string
  int 0x80
  mov rax, 1                ; syscall number for EXIT
  mov rdi, 1                ; no error
  int 0x80
