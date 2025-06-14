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

## Memory Corruption Issues in MOROS Userspace

### Critical Issue: Static Buffer Corruption

**⚠️ IMPORTANT**: MOROS userspace has a fundamental issue with static and global buffers in libc functions. This affects any function that returns pointers to static memory.

#### Problem Description

When libc functions use static or global buffers, those buffers get corrupted during function returns or shortly after. This manifests as:

1. **Function sets values correctly** inside the function
2. **Values are corrupted to zero** when returned to caller
3. **Only affects libc static/global memory** - caller-provided buffers work fine
4. **Corruption happens during function return** process

#### Affected Functions (Before Fixes)

- `gmtime()` - Returned all-zero tm structure
- `localtime()` - Returned all-zero tm structure  
- `asctime()` - Returned empty strings
- `ctime()` - Returned empty strings
- Any function using static string buffers

#### Root Cause Analysis

The exact cause is likely related to:
- **Memory layout issues** with the linker script (`-Ttext=0x800000`)
- **Stack corruption** affecting static data section
- **Calling convention problems** during function returns
- **Missing memory isolation** between userspace and kernel

#### Solution: Reentrant API Pattern

The solution is to **avoid static buffers entirely** by using reentrant APIs where the caller provides the buffer:

```c
// ❌ PROBLEMATIC: Uses static buffer
char* asctime(const struct tm* timeptr);

// ✅ SOLUTION: Caller provides buffer  
char* asctime_r(const struct tm* timeptr, char* buf);
```

#### Implementation Strategy

1. **Implement reentrant versions first** (`_r` suffix functions)
2. **Use caller-provided buffers** exclusively in reentrant versions
3. **Implement non-reentrant versions** by calling reentrant versions with static buffers
4. **Place static buffers at file/global scope** (not function-local static)

#### Working Examples

```c
// ✅ WORKING: Reentrant time functions
struct tm my_tm;
time_t timestamp = 946684801;
localtime_r(&timestamp, &my_tm);  // Works perfectly

char buffer[26];
asctime_r(&my_tm, buffer);        // Works perfectly

// ✅ WORKING: Non-reentrant versions (using reentrant internally)
struct tm* result = localtime(&timestamp);  // Now works
char* time_str = ctime(&timestamp);         // Now works
```

#### Guidelines for New libc Functions

When implementing new libc functions:

1. **Avoid static buffers** inside functions
2. **Implement reentrant versions first** (`funcname_r`)
3. **Use global buffers** if static buffers are absolutely needed
4. **Test with caller-provided buffers** to verify functionality
5. **Document any static buffer dependencies**

#### Memory Layout Considerations

- Static buffers **at file scope** are more stable than function-local static
- **Global variables** work better than static variables
- **Caller-provided buffers** (stack or heap) are most reliable
- **String literals** appear to work correctly

## Limitations

### Current Limitations

1. **File Positioning**: `fseek()`, `ftell()`, `rewind()` are not implemented
2. **String Formatting**: `sprintf()` and `snprintf()` are not fully implemented
3. **Memory Tracking**: Basic `free()` implementation without size tracking
4. **Error Handling**: Simplified error reporting
5. **Locale Support**: Not implemented
6. **Wide Character Support**: Not implemented
7. **Static Buffer Corruption**: See "Memory Corruption Issues" section above

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