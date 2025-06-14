#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include "syscall.h"

/* Environment variables - simple implementation */
static char* environment[256];
static int env_count = 0;

/* Random number generator state */
static unsigned long rand_seed = 1;

/* Exit functions */
static void (*exit_functions[32])(void);
static int exit_function_count = 0;

/* Memory management functions are in malloc.c */

/* Process control - exit is implemented in crt0.c */

void abort(void) {
    sys_exit(EXIT_FAILURE);
    __builtin_unreachable();
}

int atexit(void (*function)(void)) {
    if (!function || exit_function_count >= 32) {
        return -1;
    }
    
    exit_functions[exit_function_count++] = function;
    return 0;
}

/* Environment functions */
char* getenv(const char* name) {
    if (!name) {
        return NULL;
    }
    
    size_t name_len = strlen(name);
    
    for (int i = 0; i < env_count; i++) {
        if (environment[i]) {
            if (strncmp(environment[i], name, name_len) == 0 && 
                environment[i][name_len] == '=') {
                return environment[i] + name_len + 1;
            }
        }
    }
    
    return NULL;
}

int putenv(char* string) {
    if (!string) {
        errno = EINVAL;
        return -1;
    }
    
    char* equals = strchr(string, '=');
    if (!equals) {
        errno = EINVAL;
        return -1;
    }
    
    size_t name_len = equals - string;
    
    /* Look for existing variable */
    for (int i = 0; i < env_count; i++) {
        if (environment[i] && 
            strncmp(environment[i], string, name_len) == 0 &&
            environment[i][name_len] == '=') {
            environment[i] = string;
            return 0;
        }
    }
    
    /* Add new variable */
    if (env_count >= 255) {
        errno = ENOMEM;
        return -1;
    }
    
    environment[env_count++] = string;
    return 0;
}

int setenv(const char* name, const char* value, int overwrite) {
    if (!name || !value || strchr(name, '=')) {
        errno = EINVAL;
        return -1;
    }
    
    /* Check if variable exists */
    if (!overwrite && getenv(name)) {
        return 0;
    }
    
    /* Create name=value string */
    size_t name_len = strlen(name);
    size_t value_len = strlen(value);
    char* env_string = malloc(name_len + value_len + 2);
    if (!env_string) {
        errno = ENOMEM;
        return -1;
    }
    
    strcpy(env_string, name);
    env_string[name_len] = '=';
    strcpy(env_string + name_len + 1, value);
    
    int result = putenv(env_string);
    if (result != 0) {
        free(env_string);
    }
    
    return result;
}

int unsetenv(const char* name) {
    if (!name || strchr(name, '=')) {
        errno = EINVAL;
        return -1;
    }
    
    size_t name_len = strlen(name);
    
    for (int i = 0; i < env_count; i++) {
        if (environment[i] && 
            strncmp(environment[i], name, name_len) == 0 &&
            environment[i][name_len] == '=') {
            /* Shift remaining variables down */
            for (int j = i; j < env_count - 1; j++) {
                environment[j] = environment[j + 1];
            }
            env_count--;
            environment[env_count] = NULL;
            return 0;
        }
    }
    
    return 0;
}

/* String conversion functions */
int atoi(const char* str) {
    return (int)strtol(str, NULL, 10);
}

long atol(const char* str) {
    return strtol(str, NULL, 10);
}

long long atoll(const char* str) {
    return strtoll(str, NULL, 10);
}

/* double atof(const char* str) - disabled due to SSE issues */

/* Helper function to check if character is valid for base */
static int is_valid_digit(char c, int base) {
    if (base <= 10) {
        return c >= '0' && c < '0' + base;
    } else {
        return (c >= '0' && c <= '9') || 
               (c >= 'a' && c < 'a' + base - 10) ||
               (c >= 'A' && c < 'A' + base - 10);
    }
}

/* Helper function to convert character to digit value */
static int char_to_digit(char c) {
    if (c >= '0' && c <= '9') {
        return c - '0';
    } else if (c >= 'a' && c <= 'z') {
        return c - 'a' + 10;
    } else if (c >= 'A' && c <= 'Z') {
        return c - 'A' + 10;
    }
    return -1;
}

long strtol(const char* str, char** endptr, int base) {
    if (!str) {
        if (endptr) *endptr = (char*)str;
        errno = EINVAL;
        return 0;
    }
    
    /* Skip whitespace */
    while (*str == ' ' || *str == '\t' || *str == '\n' || 
           *str == '\r' || *str == '\f' || *str == '\v') {
        str++;
    }
    
    /* Handle sign */
    int negative = 0;
    if (*str == '-') {
        negative = 1;
        str++;
    } else if (*str == '+') {
        str++;
    }
    
    /* Handle base */
    if (base == 0) {
        if (*str == '0') {
            if (str[1] == 'x' || str[1] == 'X') {
                base = 16;
                str += 2;
            } else {
                base = 8;
                str++;
            }
        } else {
            base = 10;
        }
    } else if (base == 16 && *str == '0' && (str[1] == 'x' || str[1] == 'X')) {
        str += 2;
    }
    
    /* Convert digits */
    long result = 0;
    const char* start = str;
    
    while (*str && is_valid_digit(*str, base)) {
        int digit = char_to_digit(*str);
        if (digit >= base) break;
        
        /* Check for overflow */
        if (result > (0x7FFFFFFF - digit) / base) {
            errno = ERANGE;
            if (endptr) *endptr = (char*)str;
            return negative ? (-0x7FFFFFFF - 1) : 0x7FFFFFFF;
        }
        
        result = result * base + digit;
        str++;
    }
    
    if (str == start) {
        /* No digits converted */
        if (endptr) *endptr = (char*)start;
        return 0;
    }
    
    if (endptr) *endptr = (char*)str;
    return negative ? -result : result;
}

unsigned long strtoul(const char* str, char** endptr, int base) {
    /* Similar to strtol but for unsigned */
    long result = strtol(str, endptr, base);
    return (unsigned long)result;
}

long long strtoll(const char* str, char** endptr, int base) {
    /* Simplified - same as strtol for now */
    return (long long)strtol(str, endptr, base);
}

unsigned long long strtoull(const char* str, char** endptr, int base) {
    /* Simplified - same as strtoul for now */
    return (unsigned long long)strtoul(str, endptr, base);
}

/* Floating point conversion functions disabled due to SSE issues */
/* float strtof(const char* str, char** endptr); */
/* double strtod(const char* str, char** endptr); */
/* long double strtold(const char* str, char** endptr); */

/* Pseudo-random sequence generation */
int rand(void) {
    rand_seed = rand_seed * 1103515245 + 12345;
    return (unsigned int)(rand_seed / 65536) % 32768;
}

void srand(unsigned int seed) {
    rand_seed = seed;
}

/* Searching and sorting */
void* bsearch(const void* key, const void* base, size_t nmemb, size_t size,
              int (*compar)(const void*, const void*)) {
    if (!key || !base || !compar || nmemb == 0 || size == 0) {
        return NULL;
    }
    
    const char* array = (const char*)base;
    size_t left = 0;
    size_t right = nmemb;
    
    while (left < right) {
        size_t mid = left + (right - left) / 2;
        const void* mid_element = array + mid * size;
        
        int cmp = compar(key, mid_element);
        if (cmp == 0) {
            return (void*)mid_element;
        } else if (cmp < 0) {
            right = mid;
        } else {
            left = mid + 1;
        }
    }
    
    return NULL;
}

/* Simple bubble sort implementation for safety */
static void bubblesort(void* base, size_t nmemb, size_t size,
                      int (*compar)(const void*, const void*)) {
    if (nmemb < 2) return;
    
    char* array = (char*)base;
    char* temp = malloc(size);
    if (!temp) return;
    
    /* Bubble sort - simple but safe */
    for (size_t i = 0; i < nmemb - 1; i++) {
        for (size_t j = 0; j < nmemb - 1 - i; j++) {
            char* elem1 = array + j * size;
            char* elem2 = array + (j + 1) * size;
            
            if (compar(elem1, elem2) > 0) {
                /* Swap elements */
                memcpy(temp, elem1, size);
                memcpy(elem1, elem2, size);
                memcpy(elem2, temp, size);
            }
        }
    }
    
    free(temp);
}

void qsort(void* base, size_t nmemb, size_t size,
           int (*compar)(const void*, const void*)) {
    if (!base || !compar || nmemb == 0 || size == 0) {
        return;
    }
    
    bubblesort(base, nmemb, size, compar);
}

/* Integer arithmetic */
int abs(int j) {
    return (j < 0) ? -j : j;
}

long labs(long j) {
    return (j < 0) ? -j : j;
}

long long llabs(long long j) {
    return (j < 0) ? -j : j;
}

div_t div(int numer, int denom) {
    div_t result;
    result.quot = numer / denom;
    result.rem = numer % denom;
    return result;
}

ldiv_t ldiv(long numer, long denom) {
    ldiv_t result;
    result.quot = numer / denom;
    result.rem = numer % denom;
    return result;
}

lldiv_t lldiv(long long numer, long long denom) {
    lldiv_t result;
    result.quot = numer / denom;
    result.rem = numer % denom;
    return result;
}

/* Multibyte/wide character conversion - simplified */
int mblen(const char* s, size_t n) {
    if (!s) return 0;
    if (n == 0) return -1;
    return (*s == '\0') ? 0 : 1;
}

/* System interface - system is implemented in stdio.c */