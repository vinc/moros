#include <stdio.h>
#include <stdlib.h>
#include <stdarg.h>
#include <string.h>
#include <unistd.h>
#include "syscall.h"

/* Temporary filename constants */
#define L_tmpnam 32

/* File handles for standard streams */
static FILE _stdin = { 0, 0, 0, 0 };
static FILE _stdout = { 1, 0, 0, 0 };
static FILE _stderr = { 2, 0, 0, 0 };

FILE* stdin = &_stdin;
FILE* stdout = &_stdout;
FILE* stderr = &_stderr;

/* Open file flags mapping */
#define MOROS_OPEN_READ   0x01
#define MOROS_OPEN_WRITE  0x02
#define MOROS_OPEN_CREATE 0x04
#define MOROS_OPEN_DEVICE 0x40

/* File operations */
FILE* fopen(const char* filename, const char* mode) {
    if (!filename || !mode) {
        return NULL;
    }
    
    unsigned char flags = 0;
    
    /* Parse mode string */
    switch (mode[0]) {
        case 'r':
            flags = MOROS_OPEN_READ;
            break;
        case 'w':
            flags = MOROS_OPEN_WRITE | MOROS_OPEN_CREATE;
            break;
        case 'a':
            flags = MOROS_OPEN_WRITE | MOROS_OPEN_CREATE;
            break;
        default:
            return NULL;
    }
    
    /* Handle + modifier */
    if (mode[1] == '+') {
        flags |= MOROS_OPEN_READ | MOROS_OPEN_WRITE;
    }
    
    /* Check if this is a device file and add device flag if needed */
    if (strncmp(filename, "/dev/", 5) == 0) {
        flags |= MOROS_OPEN_DEVICE;
    }
    
    long handle = sys_open(filename, flags);
    if (handle < 0) {
        return NULL;
    }
    
    FILE* file = (FILE*)malloc(sizeof(FILE));
    if (!file) {
        sys_close((int)handle);
        return NULL;
    }
    
    file->handle = (int)handle;
    file->flags = flags;
    file->error = 0;
    file->eof = 0;
    
    return file;
}

int fclose(FILE* stream) {
    if (!stream || stream == stdin || stream == stdout || stream == stderr) {
        return EOF;
    }
    
    sys_close(stream->handle);
    free(stream);
    return 0;
}

int fflush(FILE* stream) {
    /* MOROS doesn't have explicit flush, writes are immediate */
    (void)stream;
    return 0;
}

/* Character I/O */
int fgetc(FILE* stream) {
    if (!stream) {
        return EOF;
    }
    
    unsigned char c;
    long result = sys_read(stream->handle, &c, 1);
    
    if (result <= 0) {
        if (result == 0) {
            stream->eof = 1;
        } else {
            stream->error = 1;
        }
        return EOF;
    }
    
    return (int)c;
}

int getc(FILE* stream) {
    return fgetc(stream);
}

int getchar(void) {
    return fgetc(stdin);
}

int fputc(int c, FILE* stream) {
    if (!stream) {
        return EOF;
    }
    
    unsigned char ch = (unsigned char)c;
    long result = sys_write(stream->handle, &ch, 1);
    
    if (result <= 0) {
        stream->error = 1;
        return EOF;
    }
    
    return c;
}

int putc(int c, FILE* stream) {
    return fputc(c, stream);
}

int putchar(int c) {
    return fputc(c, stdout);
}

int ungetc(int c, FILE* stream) {
    /* Not implemented - would require buffering */
    (void)c;
    (void)stream;
    return EOF;
}

/* String I/O */
char* fgets(char* s, int size, FILE* stream) {
    if (!s || size <= 0 || !stream) {
        return NULL;
    }
    
    int i = 0;
    int c;
    
    while (i < size - 1) {
        c = fgetc(stream);
        if (c == EOF) {
            if (i == 0) {
                return NULL;
            }
            break;
        }
        
        s[i++] = (char)c;
        
        if (c == '\n') {
            break;
        }
    }
    
    s[i] = '\0';
    return s;
}

char* gets(char* s) {
    /* Deprecated and unsafe - basic implementation */
    if (!s) return NULL;
    
    int i = 0;
    int c;
    
    while ((c = getchar()) != EOF && c != '\n') {
        s[i++] = (char)c;
    }
    
    if (c == EOF && i == 0) {
        return NULL;
    }
    
    s[i] = '\0';
    return s;
}

int fputs(const char* s, FILE* stream) {
    if (!s || !stream) {
        return EOF;
    }
    
    size_t len = strlen(s);
    long result = sys_write(stream->handle, s, len);
    
    if (result < 0) {
        stream->error = 1;
        return EOF;
    }
    
    return (result == (long)len) ? 0 : EOF;
}

int puts(const char* s) {
    if (fputs(s, stdout) == EOF) {
        return EOF;
    }
    
    if (fputc('\n', stdout) == EOF) {
        return EOF;
    }
    
    return 0;
}

/* Binary I/O */
size_t fread(void* ptr, size_t size, size_t nmemb, FILE* stream) {
    if (!ptr || !stream || size == 0 || nmemb == 0) {
        return 0;
    }
    
    size_t total_size = size * nmemb;
    long result = sys_read(stream->handle, ptr, total_size);
    
    if (result <= 0) {
        if (result == 0) {
            stream->eof = 1;
        } else {
            stream->error = 1;
        }
        return 0;
    }
    
    return (size_t)result / size;
}

size_t fwrite(const void* ptr, size_t size, size_t nmemb, FILE* stream) {
    if (!ptr || !stream || size == 0 || nmemb == 0) {
        return 0;
    }
    
    size_t total_size = size * nmemb;
    long result = sys_write(stream->handle, ptr, total_size);
    
    if (result <= 0) {
        stream->error = 1;
        return 0;
    }
    
    return (size_t)result / size;
}

/* Error handling */
void clearerr(FILE* stream) {
    if (stream) {
        stream->error = 0;
        stream->eof = 0;
    }
}

int feof(FILE* stream) {
    return stream ? stream->eof : 0;
}

int ferror(FILE* stream) {
    return stream ? stream->error : 0;
}

void perror(const char* s) {
    if (s && *s) {
        fputs(s, stderr);
        fputs(": ", stderr);
    }
    fputs("Error occurred\n", stderr);
}

/* Simple printf implementation */
static void print_string(const char* s, FILE* stream) {
    fputs(s, stream);
}

static void print_char(char c, FILE* stream) {
    fputc(c, stream);
}

static void print_number(long num, int base, FILE* stream) {
    if (num < 0) {
        fputc('-', stream);
        num = -num;
    }
    
    if (num == 0) {
        fputc('0', stream);
        return;
    }
    
    char buffer[32];
    int i = 0;
    
    while (num > 0) {
        int digit = num % base;
        buffer[i++] = (digit < 10) ? ('0' + digit) : ('a' + digit - 10);
        num /= base;
    }
    
    while (i > 0) {
        fputc(buffer[--i], stream);
    }
}

int vfprintf(FILE* stream, const char* format, va_list ap) {
    if (!format || !stream) {
        return -1;
    }
    
    int count = 0;
    
    while (*format) {
        if (*format != '%') {
            fputc(*format, stream);
            count++;
        } else {
            format++; /* Skip '%' */
            
            switch (*format) {
                case 'c': {
                    int c = va_arg(ap, int);
                    fputc(c, stream);
                    count++;
                    break;
                }
                case 's': {
                    const char* s = va_arg(ap, const char*);
                    if (s) {
                        print_string(s, stream);
                        count += strlen(s);
                    }
                    break;
                }
                case 'd':
                case 'i': {
                    int n = va_arg(ap, int);
                    print_number(n, 10, stream);
                    count += 10; /* Approximate */
                    break;
                }
                case 'x': {
                    unsigned int n = va_arg(ap, unsigned int);
                    print_number(n, 16, stream);
                    count += 8; /* Approximate */
                    break;
                }
                case 'l': {
                    format++; /* Skip 'l' */
                    if (*format == 'd' || *format == 'i') {
                        long n = va_arg(ap, long);
                        print_number(n, 10, stream);
                        count += 12; /* Approximate */
                    } else if (*format == 'u') {
                        unsigned long n = va_arg(ap, unsigned long);
                        print_number(n, 10, stream);
                        count += 12; /* Approximate */
                    } else if (*format == 'x') {
                        unsigned long n = va_arg(ap, unsigned long);
                        print_number(n, 16, stream);
                        count += 10; /* Approximate */
                    } else {
                        fputc('%', stream);
                        fputc('l', stream);
                        fputc(*format, stream);
                        count += 3;
                    }
                    break;
                }
                case 'p': {
                    void* ptr = va_arg(ap, void*);
                    if (ptr == NULL) {
                        print_string("(null)", stream);
                        count += 6;
                    } else {
                        fputc('0', stream);
                        fputc('x', stream);
                        print_number((unsigned long)ptr, 16, stream);
                        count += 12; /* Approximate */
                    }
                    break;
                }
                case '%':
                    fputc('%', stream);
                    count++;
                    break;
                default:
                    fputc('%', stream);
                    fputc(*format, stream);
                    count += 2;
                    break;
            }
        }
        format++;
    }
    
    return count;
}

int printf(const char* format, ...) {
    va_list ap;
    va_start(ap, format);
    int result = vfprintf(stdout, format, ap);
    va_end(ap);
    return result;
}

int fprintf(FILE* stream, const char* format, ...) {
    va_list ap;
    va_start(ap, format);
    int result = vfprintf(stream, format, ap);
    va_end(ap);
    return result;
}

int sprintf(char* str, const char* format, ...) {
    /* Simple implementation - would need custom stream for proper implementation */
    (void)str;
    (void)format;
    return -1; /* Not implemented */
}

int snprintf(char* str, size_t size, const char* format, ...) {
    /* Simple implementation - would need custom stream for proper implementation */
    (void)str;
    (void)size;
    (void)format;
    return -1; /* Not implemented */
}

/* File positioning - basic stubs */
int fseek(FILE* stream, long offset, int whence) {
    (void)stream;
    (void)offset;
    (void)whence;
    return -1; /* Not implemented */
}

long ftell(FILE* stream) {
    (void)stream;
    return -1; /* Not implemented */
}

void rewind(FILE* stream) {
    (void)stream;
    /* Not implemented */
}

/* File removal and renaming */
int remove(const char* filename) {
    if (!filename) {
        return -1;
    }
    
    /* Use unlink for file removal */
    return unlink(filename);
}

int rename(const char* old_name, const char* new_name) {
    if (!old_name || !new_name) {
        return -1;
    }
    
    /* MOROS doesn't have rename syscall yet */
    /* Would need to implement this in the kernel */
    return -1;
}

/* System command execution */
int system(const char* command) {
    if (!command) {
        return -1;
    }
    
    /* Use SYS_SPAWN to execute command */
    /* This is a simplified implementation */
    char* argv[] = { "/bin/shell", "-c", (char*)command, NULL };
    long result = sys_spawn("/bin/shell", argv);
    
    return (int)result;
}

/* Temporary file operations */
FILE* tmpfile(void) {
    /* Create a temporary file */
    /* This is a simplified implementation */
    static int tmp_counter = 0;
    char tmp_name[32];
    
    /* Simple temporary filename generation */
    int i = 0;
    const char* prefix = "/tmp/tmp";
    while (*prefix) {
        tmp_name[i++] = *prefix++;
    }
    
    /* Add counter as simple number */
    int counter = tmp_counter++;
    if (counter == 0) {
        tmp_name[i++] = '0';
    } else {
        char digits[16];
        int digit_count = 0;
        while (counter > 0) {
            digits[digit_count++] = '0' + (counter % 10);
            counter /= 10;
        }
        while (digit_count > 0) {
            tmp_name[i++] = digits[--digit_count];
        }
    }
    tmp_name[i] = '\0';
    
    return fopen(tmp_name, "w+");
}

char* tmpnam(char* s) {
    static char internal_buffer[L_tmpnam];
    static int tmp_counter = 0;
    
    char* buffer = s ? s : internal_buffer;
    
    /* Simple temporary filename generation */
    const char* prefix = "/tmp/tmp";
    int i = 0;
    while (*prefix) {
        buffer[i++] = *prefix++;
    }
    
    /* Add counter */
    int counter = tmp_counter++;
    if (counter == 0) {
        buffer[i++] = '0';
    } else {
        char digits[16];
        int digit_count = 0;
        while (counter > 0) {
            digits[digit_count++] = '0' + (counter % 10);
            counter /= 10;
        }
        while (digit_count > 0) {
            buffer[i++] = digits[--digit_count];
        }
    }
    buffer[i] = '\0';
    
    return buffer;
}

/* strlen is now implemented in syscall.h */