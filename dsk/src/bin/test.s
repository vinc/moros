[bits 64]

; Send 0 to the TEST syscall, add 1 to what we received, sleep 1s, and repeat

start:
  mov rbx, 0x0
send:
  mov rax, 0x3 ; test syscall
  mov rdi, rbx
  mov rsi, 0x0
  mov rdx, 0x0
  int 0x80
  mov rbx, rax
  add rbx, 0x1
  jmp sleep
sleep:
  mov rax, 0x0
  mov rdi, 0x3ff0000000000000
  mov rsi, 0x0
  mov rdx, 0x0
  int 0x80
  jmp send
