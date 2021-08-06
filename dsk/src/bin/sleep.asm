bits 64
start:
  mov rax, 0x0
  mov rdi, 0x3ff0000000000000
  mov rsi, 0x0
  mov rdx, 0x0
  int 0x80
  jmp start
