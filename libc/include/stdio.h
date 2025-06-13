#ifndef _STDIO_H
#define _STDIO_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* File handle structure */
typedef struct {
    int handle;
    int flags;
    int error;
    int eof;
} FILE;

/* Standard streams */
extern FILE* stdin;
extern FILE* stdout;
extern FILE* stderr;

/* File operation flags */
#define _IOFBF 0    /* full buffering */
#define _IOLBF 1    /* line buffering */
#define _IONBF 2    /* no buffering */

/* Seek origins */
#define SEEK_SET 0
#define SEEK_CUR 1
#define SEEK_END 2

/* EOF constant */
#define EOF (-1)

/* File operations */
FILE* fopen(const char* filename, const char* mode);
int fclose(FILE* stream);
int fflush(FILE* stream);

/* Character I/O */
int fgetc(FILE* stream);
int getc(FILE* stream);
int getchar(void);
int fputc(int c, FILE* stream);
int putc(int c, FILE* stream);
int putchar(int c);
int ungetc(int c, FILE* stream);

/* String I/O */
char* fgets(char* s, int size, FILE* stream);
char* gets(char* s);
int fputs(const char* s, FILE* stream);
int puts(const char* s);

/* Binary I/O */
size_t fread(void* ptr, size_t size, size_t nmemb, FILE* stream);
size_t fwrite(const void* ptr, size_t size, size_t nmemb, FILE* stream);

/* File positioning */
int fseek(FILE* stream, long offset, int whence);
long ftell(FILE* stream);
void rewind(FILE* stream);

/* Error handling */
void clearerr(FILE* stream);
int feof(FILE* stream);
int ferror(FILE* stream);
void perror(const char* s);

/* Formatted I/O */
int printf(const char* format, ...);
int fprintf(FILE* stream, const char* format, ...);
int sprintf(char* str, const char* format, ...);
int snprintf(char* str, size_t size, const char* format, ...);

int scanf(const char* format, ...);
int fscanf(FILE* stream, const char* format, ...);
int sscanf(const char* str, const char* format, ...);

/* Variable argument versions */
#include <stdarg.h>
int vprintf(const char* format, va_list ap);
int vfprintf(FILE* stream, const char* format, va_list ap);
int vsprintf(char* str, const char* format, va_list ap);
int vsnprintf(char* str, size_t size, const char* format, va_list ap);

/* File removal and renaming */
int remove(const char* filename);
int rename(const char* old_name, const char* new_name);

/* Temporary files */
FILE* tmpfile(void);
char* tmpnam(char* s);

#ifdef __cplusplus
}
#endif

#endif /* _STDIO_H */