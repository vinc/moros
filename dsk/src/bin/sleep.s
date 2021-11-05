[bits 64]
_start:
  mov rax, 9                ; syscall number for SLEEP
  mov rdi, __float64__(1.0) ; time to sleep in seconds
  mov rsi, 0
  mov rdx, 0
  int 0x80
  jmp _start
