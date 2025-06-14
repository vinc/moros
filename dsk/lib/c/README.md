# MOROS libc Implementation

A minimal C standard library implementation for the MOROS operating system.

## Overview

This libc implementation provides essential C standard library functions that interface directly with MOROS system calls. It's designed to be lightweight, efficient, and compatible with standard C programs while leveraging MOROS's unique architecture.

## Features

### Implemented Functions

#### Memory Management (`stdlib.h`)
- `malloc(size_t size)` - Allocate memory
- `calloc(size_t nmemb, size_t size)` - Allocate and zero memory
- `realloc(void* ptr, size_t size)` - Resize memory allocation
- `free(void* ptr)` - Free allocated memory
- `exit(int status)` - Terminate program

#### String Functions (`string.h`)
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
- `strtok_r(char* str, const char* delim, char** saveptr)` - Thread-safe tokenize
- `strdup(const char* s)` - Duplicate string
- Memory functions: `memcpy`, `memmove`, `memset`, `memcmp`, `memchr`

#### Input/Output (`stdio.h`)
- `printf(const char* format, ...)` - Formatted output
- `fprintf(FILE* stream, const char* format, ...)` - File formatted output
- `puts(const char* s)` - Output string with newline
- `putchar(int c)` - Output single character
- `getchar(void)` - Input single character
- `fopen(const char* filename, const char* mode)` - Open file
- `fclose(FILE* stream)` - Close file
- `fread(void* ptr, size_t size, size_t nmemb, FILE* stream)` - Read from file
- `fwrite(const void* ptr, size_t size, size_t nmemb, FILE* stream)` - Write to file
- `fgets(char* s, int size, FILE* stream)` - Read line from file
- `fputs(const char* s, FILE* stream)` - Write string to file

## Architecture

### System Call Interface

The libc communicates with MOROS through a direct system call interface defined in `src/syscall.h`. The implementation maps standard C library functions to MOROS system calls:

| C Function | MOROS Syscall | Description |
|------------|---------------|-------------|
| `malloc()` | `SYS_ALLOC` | Memory allocation |
| `free()` | `SYS_FREE` | Memory deallocation |
| `fopen()` | `SYS_OPEN` | File operations |
| `fread()`/`fwrite()` | `SYS_READ`/`SYS_WRITE` | I/O operations |
| `exit()` | `SYS_EXIT` | Process termination |

### Memory Management

The memory allocation functions (`malloc`, `calloc`, `realloc`, `free`) directly interface with MOROS's heap allocator through the `SYS_ALLOC` and `SYS_FREE` system calls. The implementation:

- Uses 8-byte alignment by default
- Handles allocation failures gracefully
- Provides standard C semantics for all allocation functions
- Could be enhanced with allocation tracking for better `free()` implementation

### File I/O

File operations are mapped to MOROS file handles:
- `stdin` → handle 0
- `stdout` → handle 1  
- `stderr` → handle 2
- Custom files → dynamically allocated handles

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

int main(int argc, char* argv[]) {
    printf("Hello, MOROS!\n");
    
    char* buffer = malloc(100);
    if (buffer) {
        strcpy(buffer, "Dynamic memory works!");
        printf("%s\n", buffer);
        free(buffer);
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

## Limitations

### Current Limitations

1. **File Positioning**: `fseek()`, `ftell()`, `rewind()` are not implemented
2. **String Formatting**: `sprintf()` and `snprintf()` are not fully implemented
3. **Memory Tracking**: Basic `free()` implementation without size tracking
4. **Error Handling**: Simplified error reporting
5. **Locale Support**: Not implemented
6. **Wide Character Support**: Not implemented

### Printf Format Support

The current `printf` implementation supports:
- `%c` - Character
- `%s` - String  
- `%d`, `%i` - Signed integer
- `%x` - Hexadecimal integer
- `%%` - Literal percent

Additional format specifiers can be added to `vfprintf()` in `src/stdio.c`.

## Testing

Test programs are provided in the `test/` directory:

```bash
# Build and run basic functionality test (from MOROS root)
make -C dsk/lib/c test
```

Example test programs in `dsk/src/c/bin/`:
- `hello.c` - Basic Hello World program
- `malloc_test.c` - Memory allocation testing

## Future Enhancements

### Planned Improvements

1. **Enhanced Printf**: Complete format specifier support
2. **File Positioning**: Implement seek/tell functions  
3. **Memory Tracking**: Better allocation tracking for proper `free()`
4. **Error Codes**: Proper errno implementation
5. **Math Functions**: Basic math library (`libm`)
6. **Time Functions**: Clock and time operations
7. **Environment**: Full environment variable support

### Advanced Features

1. **Buffered I/O**: Stream buffering for better performance
2. **Regular Expressions**: Basic regex support
3. **Networking**: Socket-like interface using MOROS network syscalls
4. **Threading**: Multi-threading support when available in MOROS

## Contributing

To contribute to the libc implementation:

1. Follow the existing code style and structure
2. Test thoroughly with provided test programs
3. Document any new functions or changes
4. Ensure compatibility with standard C semantics
5. Consider MOROS-specific optimizations

## License

This libc implementation is released under the same MIT license as MOROS.