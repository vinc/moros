/* MOROS C Runtime Startup (crt0.c) - Minimal version for testing */

/* External main function from user program */
extern int main(int argc, char* argv[]);

/* Forward declaration for exit function */
void exit(int status);

/* Entry point for C programs - simple version */
void _start(void) {
    /* For testing, use minimal arguments */
    int argc = 1;
    char* argv_array[] = {"program", 0};
    char** argv = argv_array;
    
    /* Call user's main function */
    int exit_code = main(argc, argv);
    
    /* Exit with the return code */
    exit(exit_code);
    
    /* Should never reach here */
    while(1) {
        __asm__ volatile ("hlt");
    }
}

/* Simple exit implementation that calls MOROS exit syscall */
void exit(int status) {
    /* Inline assembly to call MOROS exit syscall */
    __asm__ volatile (
        "movq $0x1, %%rax\n\t"      /* SYS_EXIT = 0x1 */
        "movq %0, %%rdi\n\t"        /* exit code */
        "int $0x80\n\t"
        :
        : "r" ((long)status)
        : "rax", "rdi", "memory"
    );
    
    /* Should never return */
    while(1) {
        __asm__ volatile ("hlt");
    }
}