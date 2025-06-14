#ifndef _UNISTD_H
#define _UNISTD_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* File access modes for access() */
#define F_OK 0  /* File exists */
#define R_OK 4  /* Read permission */
#define W_OK 2  /* Write permission */
#define X_OK 1  /* Execute permission */

/* Standard file descriptors */
#define STDIN_FILENO  0
#define STDOUT_FILENO 1
#define STDERR_FILENO 2

/* Process control */
typedef int pid_t;

/* File operations */
int access(const char* pathname, int mode);
int unlink(const char* pathname);
int rmdir(const char* pathname);
int chdir(const char* path);
char* getcwd(char* buf, size_t size);

/* Process control */
pid_t getpid(void);
pid_t getppid(void);
unsigned int sleep(unsigned int seconds);

/* I/O operations */
ssize_t read(int fd, void* buf, size_t count);
ssize_t write(int fd, const void* buf, size_t count);
int close(int fd);
int dup(int oldfd);
int dup2(int oldfd, int newfd);

/* File positioning */
off_t lseek(int fd, off_t offset, int whence);

/* Definitions for lseek */
#ifndef SEEK_SET
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2
#endif

/* Type definitions are in stddef.h */

#ifdef __cplusplus
}
#endif

#endif /* _UNISTD_H */