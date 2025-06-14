#ifndef _STDLIB_H
#define _STDLIB_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Exit status codes */
#define EXIT_SUCCESS 0
#define EXIT_FAILURE 1

/* Maximum values */
#define RAND_MAX 32767
#define SIZE_MAX ((size_t)-1)

/* Memory management */
void* malloc(size_t size);
void* calloc(size_t nmemb, size_t size);
void* realloc(void* ptr, size_t size);
void free(void* ptr);

/* Process control */
void exit(int status) __attribute__((noreturn));
void abort(void) __attribute__((noreturn));
int atexit(void (*function)(void));

/* Environment */
char* getenv(const char* name);
int putenv(char* string);
int setenv(const char* name, const char* value, int overwrite);
int unsetenv(const char* name);

/* String conversion */
int atoi(const char* str);
long atol(const char* str);
long long atoll(const char* str);
double atof(const char* str);

long strtol(const char* str, char** endptr, int base);
unsigned long strtoul(const char* str, char** endptr, int base);
long long strtoll(const char* str, char** endptr, int base);
unsigned long long strtoull(const char* str, char** endptr, int base);
float strtof(const char* str, char** endptr);
double strtod(const char* str, char** endptr);
long double strtold(const char* str, char** endptr);

/* Pseudo-random sequence generation */
int rand(void);
void srand(unsigned int seed);

/* Searching and sorting */
void* bsearch(const void* key, const void* base, size_t nmemb, size_t size,
              int (*compar)(const void*, const void*));
void qsort(void* base, size_t nmemb, size_t size,
           int (*compar)(const void*, const void*));

/* Integer arithmetic */
int abs(int j);
long labs(long j);
long long llabs(long long j);

typedef struct {
    int quot;
    int rem;
} div_t;

typedef struct {
    long quot;
    long rem;
} ldiv_t;

typedef struct {
    long long quot;
    long long rem;
} lldiv_t;

div_t div(int numer, int denom);
ldiv_t ldiv(long numer, long denom);
lldiv_t lldiv(long long numer, long long denom);

/* Multibyte/wide character conversion - simplified for now */
int mblen(const char* s, size_t n);

/* System interface */
int system(const char* command);

#ifdef __cplusplus
}
#endif

#endif /* _STDLIB_H */