# C Programming on MOROS

Welcome to C programming on MOROS! This tutorial will help you get started writing and running C programs on the MOROS operating system.

## Getting Started

MOROS includes a custom C library (libc) that provides standard C functions. You can write C programs that run natively on MOROS without needing a full operating system underneath.

## Your First C Program

Here's a simple "Hello, World!" program:

```c
#include <stdio.h>

int main(int argc, char* argv[]) {
    printf("Hello, MOROS!\n");
    return 0;
}
```

## Available Libraries

### Standard I/O (stdio.h)
- `printf()` - Print formatted text
- `puts()` - Print a string with newline
- `getchar()` - Read a single character
- `putchar()` - Write a single character

Example:
```c
#include <stdio.h>

int main() {
    printf("Enter your name: ");
    char name[50];
    // Note: scanf not available, use simple input
    printf("Hello, user!\n");
    return 0;
}
```

### Memory Management (stdlib.h)
- `malloc()` - Allocate memory
- `free()` - Free allocated memory
- `calloc()` - Allocate and zero memory
- `realloc()` - Resize memory allocation

Example:
```c
#include <stdio.h>
#include <stdlib.h>

int main() {
    // Allocate memory for 10 integers
    int* numbers = malloc(10 * sizeof(int));
    
    if (numbers == NULL) {
        printf("Memory allocation failed!\n");
        return 1;
    }
    
    // Use the memory
    for (int i = 0; i < 10; i++) {
        numbers[i] = i * i;
        printf("Square of %d is %d\n", i, numbers[i]);
    }
    
    // Always free allocated memory
    free(numbers);
    return 0;
}
```

### String Functions (string.h)
- `strlen()` - Get string length
- `strcpy()` - Copy strings
- `strcat()` - Concatenate strings
- `strcmp()` - Compare strings
- `memcpy()` - Copy memory blocks
- `memset()` - Set memory to a value

Example:
```c
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

int main() {
    char greeting[100];
    strcpy(greeting, "Hello, ");
    strcat(greeting, "MOROS!");
    
    printf("Message: %s\n", greeting);
    printf("Length: %d characters\n", (int)strlen(greeting));
    
    return 0;
}
```

## Complete Example Program

Here's a more comprehensive example that demonstrates multiple features:

```c
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char* argv[]) {
    printf("=== MOROS C Program Demo ===\n");
    
    // Memory allocation example
    printf("\n1. Memory Management:\n");
    char* buffer = malloc(256);
    if (buffer) {
        strcpy(buffer, "Dynamic memory works on MOROS!");
        printf("   %s\n", buffer);
        free(buffer);
        printf("   Memory freed successfully\n");
    }
    
    // String manipulation
    printf("\n2. String Operations:\n");
    char text[] = "MOROS is awesome";
    printf("   Original: %s\n", text);
    printf("   Length: %d\n", (int)strlen(text));
    
    // Array processing
    printf("\n3. Array Processing:\n");
    int numbers[] = {1, 4, 9, 16, 25};
    int size = sizeof(numbers) / sizeof(numbers[0]);
    
    printf("   Numbers: ");
    for (int i = 0; i < size; i++) {
        printf("%d ", numbers[i]);
    }
    printf("\n");
    
    // Command line arguments
    printf("\n4. Command Line Arguments:\n");
    printf("   Program name: %s\n", argv[0]);
    printf("   Argument count: %d\n", argc);
    
    printf("\nDemo completed successfully!\n");
    return 0;
}
```

## Programming Tips

### Memory Management
- Always check if `malloc()` returns NULL
- Every `malloc()` should have a corresponding `free()`
- Don't use memory after calling `free()`
- Use `calloc()` when you need zero-initialized memory

### String Handling
- C strings are null-terminated arrays of characters
- Always ensure string buffers are large enough
- Use `strlen()` to get string length
- Be careful with buffer overflows

### Error Handling
- Check return values from functions
- Use return codes to indicate success/failure
- Print error messages to help with debugging

### MOROS-Specific Notes
- The libc is minimal but functional
- File I/O is available through standard functions
- No floating-point math functions yet
- No complex I/O like scanf (use simple input methods)

## Running Your Programs

1. Save your C code in the filesystem
2. The build system will compile C programs automatically
3. Run your program by typing its name in the shell

Example workflow:
```
> write hello.c
[edit your code]
> hello
Hello, MOROS!
```

## Common Patterns

### Safe String Copy
```c
void safe_copy(char* dest, const char* src, int max_len) {
    int len = strlen(src);
    if (len >= max_len) {
        len = max_len - 1;
    }
    memcpy(dest, src, len);
    dest[len] = '\0';
}
```

### Dynamic Array
```c
int* create_array(int size) {
    int* arr = malloc(size * sizeof(int));
    if (arr) {
        memset(arr, 0, size * sizeof(int));
    }
    return arr;
}
```

### Simple Menu System
```c
void show_menu() {
    printf("\n=== Menu ===\n");
    printf("1. Option One\n");
    printf("2. Option Two\n");
    printf("3. Exit\n");
    printf("Choice: ");
}
```

## Next Steps

- Try the example programs in `/var/src/`
- Read the full libc documentation in `/var/doc/libc.md`
- Experiment with memory allocation and string manipulation
- Build larger programs by combining multiple concepts

Happy programming on MOROS!