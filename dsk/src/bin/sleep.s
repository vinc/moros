[bits 64]
_start:
  mov rax, 0xB              ; syscall number for SLEEP
  mov rdi, __float64__(5.0) ; time to sleep in seconds
  mov rsi, 0
  mov rdx, 0
  int 0x80
  mov rax, 0x1              ; syscall number for EXIT
  mov rdi, 0                ; no error
  int 0x80
