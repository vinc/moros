[bits 64]

        global _start

        section .text
start:  mov rax, 5                ; syscall number for WRITE
        mov rdi, 1                ; standard output
        mov rsi, msg              ; addr of string
        mov rdx, 14               ; size of string
        int 0x80
        jmp start

        section .data
	msg:  db "Hello, World!", 10
