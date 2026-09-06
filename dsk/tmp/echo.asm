code:
  mov rcx rsi          # args_len
  mov rbx rdi          # args_ptr
  jmp next             # skip first arg

loop:
  mov rsi [rbx]        # args[i].ptr
  mov rdx [rbx+8]      # args[i].len
  mov rax 0x4          # WRITE
  mov rdi 0x1          # stdout
  int 0x80

  lea rsi [space]
  mov rdx 1
  mov rax 0x4          # WRITE
  mov rdi 0x1          # stdout
  int 0x80

next:
  add rbx 16           # next arg ptr
  sub rcx 1            # decrement index
  jnz loop             # loop while non-zero

  lea rsi [newline]
  mov rdx 1
  mov rax 0x4          # WRITE
  mov rdi 0x1          # stdout

  int 0x80
  mov rax 0x1          # EXIT
  xor rdi rdi
  int 0x80

space:
  str " "

newline:
  str "\n"
