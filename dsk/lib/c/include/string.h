#ifndef _STRING_H
#define _STRING_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* String length */
size_t strlen(const char* s);

/* String copy */
char* strcpy(char* dest, const char* src);
char* strncpy(char* dest, const char* src, size_t n);

/* String concatenation */
char* strcat(char* dest, const char* src);
char* strncat(char* dest, const char* src, size_t n);

/* String comparison */
int strcmp(const char* s1, const char* s2);
int strncmp(const char* s1, const char* s2, size_t n);
int strcoll(const char* s1, const char* s2);

/* String search */
char* strchr(const char* s, int c);
char* strrchr(const char* s, int c);
char* strstr(const char* haystack, const char* needle);
char* strpbrk(const char* s, const char* accept);
size_t strspn(const char* s, const char* accept);
size_t strcspn(const char* s, const char* reject);

/* String tokenization */
char* strtok(char* str, const char* delim);
char* strtok_r(char* str, const char* delim, char** saveptr);

/* String transformation */
size_t strxfrm(char* dest, const char* src, size_t n);

/* String duplication */
char* strdup(const char* s);
char* strndup(const char* s, size_t n);

/* Memory functions */
void* memcpy(void* dest, const void* src, size_t n);
void* memmove(void* dest, const void* src, size_t n);
void* memset(void* s, int c, size_t n);
int memcmp(const void* s1, const void* s2, size_t n);
void* memchr(const void* s, int c, size_t n);

/* Error string */
char* strerror(int errnum);

/* Case conversion helpers (non-standard but useful) */
char* strupper(char* s);
char* strlower(char* s);

/* Safe string functions (non-standard but recommended) */
size_t strlcpy(char* dest, const char* src, size_t size);
size_t strlcat(char* dest, const char* src, size_t size);

#ifdef __cplusplus
}
#endif

#endif /* _STRING_H */