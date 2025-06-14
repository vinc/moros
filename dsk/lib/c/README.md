# MOROS libc Implementation

A C standard library implementation for the MOROS operating system.

## Overview

This libc implementation provides essential C standard library functions that
interface directly with MOROS system calls. It's designed to be lightweight,
efficient, and compatible with standard C programs while leveraging MOROS's
unique architecture.

Note: MOROS is not UNIX-based and is not intended to be fully POSIX-compatible.
MOROS uses its own system call interface (e.g., spawn instead of fork/exec) and
architectural decisions optimized for simplicity and performance.

## Implemented Functions

### Memory Management (stdlib.h)
- `malloc(size_t size)` - Allocate memory
- `calloc(size_t nmemb, size_t size)` - Allocate and zero memory
- `realloc(void* ptr, size_t size)` - Resize memory allocation
- `free(void* ptr)` - Free allocated memory
- `exit(int status)` - Terminate program

### String Functions (string.h)
- `strlen(const char* s)` - Get string length
- `strcpy(char* dest, const char* src)` - Copy string
- `strncpy(char* dest, const char* src, size_t n)` - Copy string with limit
- `strcat(char* dest, const char* src)` - Concatenate strings
- `strncat(char* dest, const char* src, size_t n)` - Concatenate with limit
- `strcmp(const char* s1, const char* s2)` - Compare strings
- `strncmp(const char* s1, const char* s2, size_t n)` - Compare with limit
- `strchr(const char* s, int c)` - Find character in string
- `strrchr(const char* s, int c)` - Find last occurrence of character
- `strstr(const char* haystack, const char* needle)` - Find substring
- `strtok(char* str, const char* delim)` - Tokenize string
- `strtok_r(char* str, const char* delim, char** saveptr)`
- `strdup(const char* s)` - Duplicate string
- Memory functions: `memcpy`, `memmove`, `memset`, `memcmp`, `memchr`

### Input/Output (stdio.h)
- `printf(const char* format, ...)` - Formatted output
- `fprintf(FILE* stream, const char* format, ...)` - File formatted output
- `puts(const char* s)` - Output string with newline
- `putchar(int c)` - Output single character
- `getchar(void)` - Input single character
- `fopen(const char* filename, const char* mode)` - Open file
- `fclose(FILE* stream)` - Close file
- `fread(void* ptr, size_t size, size_t nmemb, FILE* stream)` - Read from file
- `fwrite(const void* ptr, size_t size, size_t nmemb, FILE* stream)` - Write
- `fgets(char* s, int size, FILE* stream)` - Read line from file
- `fputs(const char* s, FILE* stream)` - Write string to file

### Directory Operations (dirent.h)
- `opendir(const char* name)` - Open directory stream
- `readdir(DIR* dirp)` - Read directory entry
- `readdir_r(DIR* dirp, struct dirent* entry, struct dirent** result)`
- `closedir(DIR* dirp)` - Close directory stream
- `rewinddir(DIR* dirp)` - Rewind directory stream
- `telldir(DIR* dirp)` - Get current position in directory
- `seekdir(DIR* dirp, long loc)` - Seek to position in directory
- `dirfd(DIR* dirp)` - Get file descriptor from directory stream
- `fdopendir(int fd)` - Create directory stream from file descriptor
- `scandir(const char* dirp, struct dirent*** namelist, ...)` - Scan directory
- `alphasort(const struct dirent** a, const struct dirent** b)` - Alphabetical sort
- `versionsort(const struct dirent** a, const struct dirent** b)` - Version sort

### File Status and Permissions (sys/stat.h)
- `stat(const char* pathname, struct stat* buf)` - Get file status
- `fstat(int fd, struct stat* buf)` - Get file status from descriptor
- `lstat(const char* pathname, struct stat* buf)` - Get file status (no symlinks)
- `mkdir(const char* pathname, mode_t mode)` - Create directory
- `chmod(const char* pathname, mode_t mode)` - Change file permissions
- `fchmod(int fd, mode_t mode)` - Change file permissions via descriptor
- `umask(mode_t mask)` - Set file mode creation mask

### Time and Date Functions (time.h)
- `clock(void)` - Get processor time
- `time(time_t* tloc)` - Get current time
- `difftime(time_t time1, time_t time0)` - Calculate time difference
- `mktime(struct tm* timeptr)` - Convert tm structure to time_t
- `gmtime(const time_t* timer)` - Convert time_t to UTC tm structure
- `gmtime_r(const time_t* timer, struct tm* result)`
- `localtime(const time_t* timer)` - Convert time_t to local tm structure
- `localtime_r(const time_t* timer, struct tm* result)`
- `asctime(const struct tm* timeptr)` - Convert tm to string
- `asctime_r(const struct tm* timeptr, char* buf)`
- `ctime(const time_t* timer)` - Convert time_t to string
- `strftime(char* s, size_t maxsize, const char* format, const struct tm* timeptr)`

### System Interface (unistd.h)
- `access(const char* pathname, int mode)` - Check file access permissions
- `unlink(const char* pathname)` - Delete a file
- `rmdir(const char* pathname)` - Remove a directory
- `chdir(const char* path)` - Change current directory
- `getcwd(char* buf, size_t size)` - Get current working directory
- `getpid(void)` - Get process ID
- `getppid(void)` - Get parent process ID
- `sleep(unsigned int seconds)` - Sleep for specified seconds
- `read(int fd, void* buf, size_t count)` - Read from file descriptor
- `write(int fd, const void* buf, size_t count)` - Write to file descriptor
- `close(int fd)` - Close file descriptor
- `dup(int oldfd)` - Duplicate file descriptor
- `dup2(int oldfd, int newfd)` - Duplicate file descriptor to specific fd
- `lseek(int fd, off_t offset, int whence)` - Seek in file

### Path Manipulation (libgen.h)
- `basename(char* path)` - Extract filename from path
- `dirname(char* path)` - Extract directory from path

### Error Handling (errno.h)
- `errno` - Global error variable
- `strerror(int errnum)` - Convert error number to string

## Architecture

### MOROS System Call Interface

The libc communicates with MOROS through a direct system call interface
defined in src/syscall.h. The implementation maps standard C library functions
to MOROS-specific system calls:

| C Function           | MOROS Syscall | Description             |
|----------------------|---------------|-------------------------|
| `malloc()`           | SYS_ALLOC     | Memory allocation       |
| `free()`             | SYS_FREE      | Memory deallocation     |
| `fopen()`            | SYS_OPEN      | File operations         |
| `fread()`/`fwrite()` | SYS_READ/WRITE| I/O operations          |
| `exit()`             | SYS_EXIT      | Process termination     |
| `stat()`             | SYS_INFO      | File information        |
| `unlink()`           | SYS_DELETE    | File deletion           |
| `sleep()`            | SYS_SLEEP     | Process suspension      |

### Memory Management

The memory allocation functions (`malloc`, `calloc`, `realloc`, `free`) directly
interface with MOROS's heap allocator through the SYS_ALLOC and SYS_FREE
system calls. The implementation:

- Uses 8-byte alignment by default
- Handles allocation failures gracefully
- Provides standard C semantics for all allocation functions
- Could be enhanced with allocation tracking for better `free()` implementation

### File I/O

File operations are mapped to MOROS file handles:
- `stdin` -> handle 0
- `stdout` -> handle 1
- `stderr` -> handle 2
- Custom files -> dynamically allocated handles

Directory operations use MOROS's directory listing format, parsing 14-byte
metadata headers followed by filename data.

## Building

### Prerequisites

- Clang compiler with x86_64 target support
- GNU Make
- MOROS development environment

### Build Commands

```bash
# Build the libc library (from MOROS root)
make libc

# Clean build artifacts (from MOROS root)
make libc-clean

# Build C programs (from MOROS root)
make user-c

# Build everything including libc
make user
```

### Integration with MOROS

The main MOROS Makefile automatically builds libc and C programs:

```bash
# Build MOROS with C program support
make image

# This will:
# 1. Build the libc library from dsk/lib/c/
# 2. Compile any C programs in dsk/src/c/bin/
# 3. Package them into the MOROS filesystem
```

## Usage

### Writing C Programs for MOROS

Create C source files in `dsk/src/c/bin/` directory:

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <dirent.h>

int main(int argc, char* argv[]) {
    printf("Hello, MOROS!\n");
    
    // Memory allocation
    char* buffer = malloc(100);
    if (buffer) {
        strcpy(buffer, "Dynamic memory works!");
        printf("%s\n", buffer);
        free(buffer);
    }
    
    // Time functions
    time_t now = time(NULL);
    printf("Current time: %s", ctime(&now));
    
    // Directory listing
    DIR* dir = opendir("/");
    if (dir) {
        struct dirent* entry;
        while ((entry = readdir(dir)) != NULL) {
            printf("File: %s\n", entry->d_name);
        }
        closedir(dir);
    }
    
    return 0;
}
```

### Compilation Flags

C programs are compiled with these flags:
- `-target x86_64-unknown-none` - Bare metal x86_64 target
- `-ffreestanding` - Freestanding environment
- `-nostdlib` - Don't link standard libraries
- `-fno-stack-protector` - Disable stack protection
- `-mno-red-zone` - Required for kernel compatibility
- `-fno-builtin` - Don't use compiler builtins

## Memory Corruption Issues in MOROS Userspace

### Critical Issue: Static Buffer Corruption

WARNING: MOROS userspace has a fundamental issue with static and global buffers
in libc functions. This affects any function that returns pointers to static
memory.

#### Problem Description

When libc functions use static or global buffers, those buffers get corrupted
during function returns or shortly after. This manifests as:

1. Function sets values correctly inside the function
2. Values are corrupted to zero when returned to caller
3. Only affects libc static/global memory - caller-provided buffers work fine
4. Corruption happens during function return process

#### Solution: Reentrant API Pattern

The solution is to avoid static buffers entirely by using reentrant APIs where
the caller provides the buffer:

```c
// PROBLEMATIC: Uses static buffer
char* asctime(const struct tm* timeptr);

// SOLUTION: Caller provides buffer
char* asctime_r(const struct tm* timeptr, char* buf);
```

#### Implementation Strategy

1. Implement reentrant versions first (`_r` suffix functions)
2. Use caller-provided buffers exclusively in reentrant versions
3. Implement non-reentrant versions by calling reentrant versions with global
   buffers
4. Place static buffers at file/global scope (not function-local static)

#### Working Examples

```c
// WORKING: Reentrant time functions
struct tm my_tm;
time_t timestamp = 946684801;
localtime_r(&timestamp, &my_tm);  // Works perfectly

char buffer[26];
asctime_r(&my_tm, buffer);        // Works perfectly

// WORKING: Non-reentrant versions (using reentrant internally)
struct tm* result = localtime(&timestamp);  // Now works
char* time_str = ctime(&timestamp);         // Now works
```

## Limitations

### Current Limitations

1. File Positioning: Limited `lseek()` support (MOROS filesystem limitation)
2. String Formatting: `sprintf()` and `snprintf()` are not fully implemented
3. Process Control: No `fork()`/`exec()` - MOROS uses `spawn()` instead
4. Signals: No POSIX signal handling (not applicable to MOROS)
5. Networking: Not implemented (would use MOROS network syscalls)
6. Locale Support: Not implemented
7. Wide Character Support: Not implemented
8. Static Buffer Corruption: See "Memory Corruption Issues" section above

### Printf Format Support

The current `printf` implementation supports:
- `%c` - Character
- `%s` - String
- `%d`, `%i` - Signed integer
- `%x` - Hexadecimal integer
- `%%` - Literal percent

Additional format specifiers can be added to `vfprintf()` in `src/stdio.c`.

### MOROS-Specific Design Decisions

- No POSIX Compliance: Functions are implemented for compatibility, not strict
  POSIX adherence
- Simplified Error Handling: Error codes are mapped to approximate POSIX
  equivalents
- No Multi-User Support: File permissions are simplified
- No Process Hierarchy: PID functions return simplified values

## Testing

Test programs are provided in the `test/` directory:

```bash
# Build and run basic functionality test (from MOROS root)
make -C dsk/lib/c test
```

Example test programs in `dsk/src/c/bin/`:
- `hello.c` - Basic Hello World program
- `malloc_test.c` - Memory allocation testing
- `time_test.c` - Time function testing
- `dir_test.c` - Directory operation testing

## License

This libc implementation is released under the same MIT license as MOROS.
