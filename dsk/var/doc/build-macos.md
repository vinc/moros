# Building MOROS with libc Support on macOS

This guide explains how to build MOROS with C library support on macOS.

## Prerequisites

### Required Tools

1. **Xcode Command Line Tools**:
   ```bash
   xcode-select --install
   ```

2. **Homebrew** (if not already installed):
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

3. **Cross-compilation toolchain**:
   ```bash
   brew install x86_64-elf-binutils
   ```

4. **QEMU** (for testing):
   ```bash
   brew install qemu
   ```

5. **Rust toolchain** (follow MOROS setup):
   ```bash
   make setup
   ```

### Verify Installation

Check that the cross-compilation tools are available:
```bash
which x86_64-elf-ld
which x86_64-elf-ar
which x86_64-elf-objcopy
x86_64-elf-ld --version
```

## Building

### 1. Build Everything (Recommended)
```bash
make image
```
This will:
- Build the libc library
- Compile NASM assembly programs
- Compile Rust userspace programs
- Compile C programs using libc
- Build the MOROS kernel
- Create the bootable disk image

### 2. Build Individual Components

**Build only the libc library**:
```bash
make libc
```

**Build only C programs**:
```bash
make user-c
```

**Build all userspace programs**:
```bash
make user
```

**Clean everything**:
```bash
make clean
```

## Running MOROS

Start MOROS in QEMU:
```bash
make qemu
```

To run with different options:
```bash
# Run with serial output
make qemu output=serial

# Run with different memory size
make qemu memory=64

# Run with different network card
make qemu nic=e1000
```

## Writing C Programs

### 1. Create C Source Files

Place your C programs in `dsk/src/c/bin/`:

```c
// dsk/src/c/bin/myprogram.c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char* argv[]) {
    printf("Hello from C on MOROS!\n");
    
    char* buffer = malloc(100);
    if (buffer) {
        strcpy(buffer, "Memory allocation works!");
        printf("%s\n", buffer);
        free(buffer);
    }
    
    return 0;
}
```

### 2. Build and Test

```bash
# Build C programs
make user-c

# Build everything and create disk image
make image

# Run in QEMU
make qemu
```

### 3. Available libc Functions

The MOROS libc implementation includes:

**Memory Management**:
- `malloc()`, `calloc()`, `realloc()`, `free()`

**String Functions**:
- `strlen()`, `strcpy()`, `strcat()`, `strcmp()`
- `memcpy()`, `memset()`, `memcmp()`
- `strtok()`, `strdup()`

**Input/Output**:
- `printf()`, `fprintf()`, `puts()`
- `fopen()`, `fclose()`, `fread()`, `fwrite()`
- `getchar()`, `putchar()`

**Process Control**:
- `exit()`

## Troubleshooting

### Common Issues

1. **Linker errors about unknown options**:
   - Make sure `x86_64-elf-binutils` is installed
   - Verify the cross-compilation tools are in your PATH

2. **"command not found" errors**:
   ```bash
   # Add Homebrew tools to PATH
   export PATH="/usr/local/bin:$PATH"
   ```

3. **Build fails with missing files**:
   ```bash
   # Create missing placeholder files
   touch dsk/bin/ntp dsk/bin/pkg
   
   # Or run website build
   make website
   ```

4. **Strip warnings**:
   - The warnings about strip not processing MOROS binaries are normal
   - MOROS uses a custom binary format that macOS strip doesn't understand

### Clean Build

If you encounter persistent issues:
```bash
make clean
make website  # Recreate web files
make image    # Full rebuild
```

## Architecture Notes

### Cross-Compilation Setup

MOROS requires cross-compilation because:
- Target: `x86_64-unknown-none` (bare metal)
- Host: `aarch64-apple-darwin` or `x86_64-apple-darwin`

The build system automatically detects macOS and uses:
- `x86_64-elf-ld` instead of system `ld`
- `x86_64-elf-ar` instead of system `ar`
- `x86_64-elf-objcopy` for binary conversion

### libc Implementation

The MOROS libc:
- Interfaces directly with MOROS system calls
- Provides standard C library functions
- Uses a custom C runtime (`crt0.c`) for program startup
- Creates MOROS-compatible binaries with magic header `\x7FBIN`

### Binary Format

C programs are compiled as:
1. C source → object file (`.o`)
2. Link with libc → ELF executable
3. Convert to raw binary
4. Add MOROS magic header → MOROS binary

## Development Tips

1. **Debugging C programs**: Use printf for output and simple debugging
2. **Memory management**: Always pair `malloc()` with `free()`
3. **Error handling**: Check return values from libc functions
4. **Performance**: The libc is minimal - avoid complex operations

## Next Steps

- Explore example programs in `dsk/src/c/bin/`
- Read the libc documentation in `dsk/lib/c/README.md`
- Try porting simple C programs to MOROS
- Contribute improvements to the libc implementation