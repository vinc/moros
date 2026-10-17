code:
  mov eax 0x4    # WRITE syscall
  mov edi 0x1    # stdout
  lea rsi [data] # addr of str
  mov edx 14     # size of str
  int 0x80
  mov eax 0x1    # EXIT syscall
  mov edi 0x0    # success
  int 0x80

data:
  str "Hello, World!\n"
