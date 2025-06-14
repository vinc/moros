#ifndef _SYSCALL_H
#define _SYSCALL_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Forward declaration for strlen - implemented in string.c */
extern size_t strlen(const char* s);

/* MOROS syscall numbers - matching moros/src/sys/syscall/number.rs */
#define SYS_EXIT    0x1
#define SYS_SPAWN   0x2
#define SYS_READ    0x3
#define SYS_WRITE   0x4
#define SYS_OPEN    0x5
#define SYS_CLOSE   0x6
#define SYS_INFO    0x7
#define SYS_DUP     0x8
#define SYS_DELETE  0x9
#define SYS_STOP    0xA
#define SYS_SLEEP   0xB
#define SYS_POLL    0xC
#define SYS_CONNECT 0xD
#define SYS_LISTEN  0xE
#define SYS_ACCEPT  0xF
#define SYS_ALLOC   0x10
#define SYS_FREE    0x11
#define SYS_KIND    0x12

/* Syscall wrapper - MOROS uses int 0x80 */
static inline long syscall(long number, long arg1, long arg2, long arg3, long arg4, long arg5, long arg6) {
    long result;
    (void)arg4; (void)arg5; (void)arg6; /* Unused for now */
    
    __asm__ volatile ("int $0x80"
                      : "=a" (result)
                      : "a" (number), "D" (arg1), "S" (arg2), "d" (arg3)
                      : "memory");
    
    return result;
}

/* Convenience macros for different numbers of arguments */
#define syscall0(n) syscall(n, 0, 0, 0, 0, 0, 0)
#define syscall1(n, a1) syscall(n, (long)(a1), 0, 0, 0, 0, 0)
#define syscall2(n, a1, a2) syscall(n, (long)(a1), (long)(a2), 0, 0, 0, 0)
#define syscall3(n, a1, a2, a3) syscall(n, (long)(a1), (long)(a2), (long)(a3), 0, 0, 0)
#define syscall4(n, a1, a2, a3, a4) syscall(n, (long)(a1), (long)(a2), (long)(a3), (long)(a4), 0, 0)
#define syscall5(n, a1, a2, a3, a4, a5) syscall(n, (long)(a1), (long)(a2), (long)(a3), (long)(a4), (long)(a5), 0)
#define syscall6(n, a1, a2, a3, a4, a5, a6) syscall(n, (long)(a1), (long)(a2), (long)(a3), (long)(a4), (long)(a5), (long)(a6))

/* MOROS-specific syscall wrappers */
static inline void sys_exit(int code) {
    syscall1(SYS_EXIT, code);
    __builtin_unreachable();
}

static inline long sys_read(int handle, void* buf, size_t count) {
    return syscall3(SYS_READ, handle, buf, count);
}

static inline long sys_write(int handle, const void* buf, size_t count) {
    return syscall3(SYS_WRITE, handle, buf, count);
}

static inline long sys_open(const char* path, unsigned char flags) {
    return syscall3(SYS_OPEN, path, strlen(path), flags);
}

static inline void sys_close(int handle) {
    syscall1(SYS_CLOSE, handle);
}

static inline void* sys_alloc(size_t size, size_t align) {
    return (void*)syscall2(SYS_ALLOC, size, align);
}

static inline void sys_free(void* ptr, size_t size, size_t align) {
    syscall3(SYS_FREE, ptr, size, align);
}

static inline long sys_spawn(const char* path, char* const argv[]) {
    size_t path_len = strlen(path);
    size_t argc = 0;
    
    /* Count arguments */
    if (argv) {
        while (argv[argc]) argc++;
    }
    
    return syscall4(SYS_SPAWN, path, path_len, argv, argc);
}

static inline long sys_info(const char* path, void* info_buf) {
    return syscall3(SYS_INFO, path, strlen(path), info_buf);
}

#ifdef __cplusplus
}
#endif

#endif /* _SYSCALL_H */