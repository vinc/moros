[bits 64]
start:
  mov rax, 0x0                ; sleep syscall
  mov rdi, 0x3ff0000000000000 ; 1.0
  mov rsi, 0x0
  mov rdx, 0x0
  int 0x80
  jmp start
