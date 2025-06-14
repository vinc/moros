#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <errno.h>
#include "syscall.h"

/* Global errno variable is defined in errno.c */
extern int errno;

/* Check file access permissions */
int access(const char* pathname, int mode) {
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    
    /* Use SYS_INFO to check if file exists - this is the proper way in MOROS */
    struct {
        unsigned char kind;      /* File type */
        unsigned char reserved[3];
        unsigned int size;       /* File size */
        unsigned int time;       /* Timestamp */
        unsigned char name_len;  /* Filename length */
        char name[256];          /* Filename */
    } info;
    
    long result = sys_info(pathname, &info);
    if (result < 0) {
        errno = ENOENT;
        return -1;
    }
    
    /* For MOROS, we'll assume all existing files are readable */
    /* This is a simplified implementation */
    if (mode == F_OK || mode == R_OK) {
        return 0;
    }
    
    /* For write/execute, we'd need more sophisticated checks */
    /* For now, assume success for simplicity */
    return 0;
}

/* Delete a file */
int unlink(const char* pathname) {
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    
    /* Use SYS_DELETE syscall */
    long result = syscall1(SYS_DELETE, pathname);
    if (result < 0) {
        errno = ENOENT;
        return -1;
    }
    
    return 0;
}

/* Remove a directory */
int rmdir(const char* pathname) {
    /* For MOROS, same as unlink for now */
    return unlink(pathname);
}

/* Change current directory */
int chdir(const char* path) {
    if (!path) {
        errno = EINVAL;
        return -1;
    }
    
    /* MOROS doesn't have a direct chdir syscall yet */
    /* We'd need to add this functionality */
    /* For now, return success but don't actually change directory */
    errno = ENOSYS;
    return -1;
}

/* Get current working directory */
char* getcwd(char* buf, size_t size) {
    const char* cwd = "/"; /* Default to root for MOROS */
    size_t len = strlen(cwd);
    
    if (buf == NULL) {
        if (size == 0) {
            size = len + 1;
        }
        buf = malloc(size);
        if (!buf) {
            errno = ENOMEM;
            return NULL;
        }
    }
    
    if (size <= len) {
        errno = ERANGE;
        return NULL;
    }
    
    strcpy(buf, cwd);
    return buf;
}

/* Get process ID */
pid_t getpid(void) {
    /* MOROS doesn't have traditional PIDs yet */
    return 1;
}

/* Get parent process ID */
pid_t getppid(void) {
    /* MOROS doesn't have traditional PIDs yet */
    return 0;
}

/* Sleep for specified seconds */
unsigned int sleep(unsigned int seconds) {
    syscall1(SYS_SLEEP, *(unsigned long*)&(double){(double)seconds});
    return 0;
}

/* Read from file descriptor */
ssize_t read(int fd, void* buf, size_t count) {
    if (!buf) {
        errno = EINVAL;
        return -1;
    }
    
    long result = sys_read(fd, buf, count);
    if (result < 0) {
        errno = EIO;
        return -1;
    }
    
    return (ssize_t)result;
}

/* Write to file descriptor */
ssize_t write(int fd, const void* buf, size_t count) {
    if (!buf) {
        errno = EINVAL;
        return -1;
    }
    
    long result = sys_write(fd, buf, count);
    if (result < 0) {
        errno = EIO;
        return -1;
    }
    
    return (ssize_t)result;
}

/* Close file descriptor */
int close(int fd) {
    sys_close(fd);
    return 0;
}

/* Duplicate file descriptor */
int dup(int oldfd) {
    /* Use SYS_DUP syscall */
    long result = syscall2(SYS_DUP, oldfd, 0);
    if (result < 0) {
        errno = EBADF;
        return -1;
    }
    return (int)result;
}

/* Duplicate file descriptor to specific fd */
int dup2(int oldfd, int newfd) {
    if (oldfd == newfd) {
        return newfd;
    }
    
    /* Close new fd if it's open */
    close(newfd);
    
    /* Use SYS_DUP syscall */
    long result = syscall2(SYS_DUP, oldfd, newfd);
    if (result < 0) {
        errno = EBADF;
        return -1;
    }
    return newfd;
}

/* Seek in file */
off_t lseek(int fd, off_t offset, int whence) {
    /* MOROS doesn't have seek functionality yet */
    /* This would need to be implemented in the kernel */
    errno = ENOSYS;
    return -1;
}