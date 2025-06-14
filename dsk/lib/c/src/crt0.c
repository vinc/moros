/* MOROS C Runtime Startup (crt0.c) - Minimal version for testing */

#include <stddef.h>

/* External main function from user program */
extern int main(int argc, char* argv[]);

/* Forward declaration for exit function */
void exit(int status);

/* Inline sys_write using inline assembly */
static void debug_write(const char* msg, int len) {
    __asm__ volatile (
        "movq $0x4, %%rax\n\t"      /* SYS_WRITE = 0x4 */
        "movq $1, %%rdi\n\t"        /* stdout */
        "movq %0, %%rsi\n\t"        /* message */
        "movq %1, %%rdx\n\t"        /* length */
        "int $0x80\n\t"
        :
        : "r" (msg), "r" ((long)len)
        : "rax", "rdi", "rsi", "rdx", "memory"
    );
}

/* Entry point for C programs - receives arguments from MOROS */
void _start(void* args_ptr, unsigned long args_len) {
    int argc = (int)args_len;
    
    /* MOROS passes an array of Rust &str objects */
    /* Each &str is a fat pointer: {data_ptr, length} */
    void** rust_str_array = (void**)args_ptr;
    
    /* Allocate argv array on stack */
    char** argv = (char**)__builtin_alloca(argc * sizeof(char*));
    
    /* Convert each Rust &str to C null-terminated string */
    for (int i = 0; i < argc; i++) {
        /* Each Rust &str is two consecutive pointers: data and length */
        char* str_data = (char*)rust_str_array[i * 2];
        size_t str_len = (size_t)rust_str_array[i * 2 + 1];
        
        /* Allocate space for C string (with null terminator) */
        char* c_str = (char*)__builtin_alloca(str_len + 1);
        
        /* Copy string data */
        for (size_t j = 0; j < str_len; j++) {
            c_str[j] = str_data[j];
        }
        c_str[str_len] = '\0';  /* Null terminate */
        
        argv[i] = c_str;
    }
    
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