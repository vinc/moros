#include <string.h>
#include <stdlib.h>

/* String length */
size_t strlen(const char* s) {
    if (!s) return 0;
    
    size_t len = 0;
    while (s[len]) {
        len++;
    }
    return len;
}

/* String copy */
char* strcpy(char* dest, const char* src) {
    if (!dest || !src) return dest;
    
    char* d = dest;
    while ((*d++ = *src++));
    return dest;
}

char* strncpy(char* dest, const char* src, size_t n) {
    if (!dest || !src) return dest;
    
    size_t i;
    for (i = 0; i < n && src[i]; i++) {
        dest[i] = src[i];
    }
    for (; i < n; i++) {
        dest[i] = '\0';
    }
    return dest;
}

/* String concatenation */
char* strcat(char* dest, const char* src) {
    if (!dest || !src) return dest;
    
    char* d = dest + strlen(dest);
    while ((*d++ = *src++));
    return dest;
}

char* strncat(char* dest, const char* src, size_t n) {
    if (!dest || !src) return dest;
    
    char* d = dest + strlen(dest);
    size_t i;
    for (i = 0; i < n && src[i]; i++) {
        d[i] = src[i];
    }
    d[i] = '\0';
    return dest;
}

/* String comparison */
int strcmp(const char* s1, const char* s2) {
    if (!s1 || !s2) {
        if (s1 == s2) return 0;
        return s1 ? 1 : -1;
    }
    
    while (*s1 && (*s1 == *s2)) {
        s1++;
        s2++;
    }
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

int strncmp(const char* s1, const char* s2, size_t n) {
    if (!s1 || !s2 || n == 0) {
        if (n == 0) return 0;
        if (s1 == s2) return 0;
        return s1 ? 1 : -1;
    }
    
    while (n > 0 && *s1 && (*s1 == *s2)) {
        s1++;
        s2++;
        n--;
    }
    
    if (n == 0) return 0;
    return *(unsigned char*)s1 - *(unsigned char*)s2;
}

int strcoll(const char* s1, const char* s2) {
    /* Simple implementation - just use strcmp */
    return strcmp(s1, s2);
}

/* String search */
char* strchr(const char* s, int c) {
    if (!s) return NULL;
    
    while (*s) {
        if (*s == (char)c) {
            return (char*)s;
        }
        s++;
    }
    
    if (c == '\0') {
        return (char*)s;
    }
    
    return NULL;
}

char* strrchr(const char* s, int c) {
    if (!s) return NULL;
    
    const char* last = NULL;
    
    while (*s) {
        if (*s == (char)c) {
            last = s;
        }
        s++;
    }
    
    if (c == '\0') {
        return (char*)s;
    }
    
    return (char*)last;
}

char* strstr(const char* haystack, const char* needle) {
    if (!haystack || !needle) return NULL;
    
    if (!*needle) return (char*)haystack;
    
    size_t needle_len = strlen(needle);
    
    while (*haystack) {
        if (strncmp(haystack, needle, needle_len) == 0) {
            return (char*)haystack;
        }
        haystack++;
    }
    
    return NULL;
}

char* strpbrk(const char* s, const char* accept) {
    if (!s || !accept) return NULL;
    
    while (*s) {
        const char* a = accept;
        while (*a) {
            if (*s == *a) {
                return (char*)s;
            }
            a++;
        }
        s++;
    }
    
    return NULL;
}

size_t strspn(const char* s, const char* accept) {
    if (!s || !accept) return 0;
    
    size_t count = 0;
    
    while (*s) {
        const char* a = accept;
        int found = 0;
        
        while (*a) {
            if (*s == *a) {
                found = 1;
                break;
            }
            a++;
        }
        
        if (!found) break;
        
        s++;
        count++;
    }
    
    return count;
}

size_t strcspn(const char* s, const char* reject) {
    if (!s || !reject) return s ? strlen(s) : 0;
    
    size_t count = 0;
    
    while (*s) {
        const char* r = reject;
        
        while (*r) {
            if (*s == *r) {
                return count;
            }
            r++;
        }
        
        s++;
        count++;
    }
    
    return count;
}

/* String tokenization */
static char* strtok_state = NULL;

char* strtok(char* str, const char* delim) {
    return strtok_r(str, delim, &strtok_state);
}

char* strtok_r(char* str, const char* delim, char** saveptr) {
    if (!delim || !saveptr) return NULL;
    
    if (str) {
        *saveptr = str;
    } else if (!*saveptr) {
        return NULL;
    }
    
    /* Skip leading delimiters */
    *saveptr += strspn(*saveptr, delim);
    
    if (!**saveptr) {
        *saveptr = NULL;
        return NULL;
    }
    
    char* token = *saveptr;
    *saveptr += strcspn(*saveptr, delim);
    
    if (**saveptr) {
        **saveptr = '\0';
        (*saveptr)++;
    } else {
        *saveptr = NULL;
    }
    
    return token;
}

/* String transformation */
size_t strxfrm(char* dest, const char* src, size_t n) {
    /* Simple implementation - just copy */
    if (!src) return 0;
    
    size_t src_len = strlen(src);
    
    if (dest && n > 0) {
        strncpy(dest, src, n - 1);
        dest[n - 1] = '\0';
    }
    
    return src_len;
}

/* String duplication */
char* strdup(const char* s) {
    if (!s) return NULL;
    
    size_t len = strlen(s) + 1;
    char* copy = malloc(len);
    
    if (copy) {
        memcpy(copy, s, len);
    }
    
    return copy;
}

char* strndup(const char* s, size_t n) {
    if (!s) return NULL;
    
    size_t len = strlen(s);
    if (len > n) len = n;
    
    char* copy = malloc(len + 1);
    if (copy) {
        memcpy(copy, s, len);
        copy[len] = '\0';
    }
    
    return copy;
}

/* Memory functions */
void* memcpy(void* dest, const void* src, size_t n) {
    if (!dest || !src) return dest;
    
    char* d = (char*)dest;
    const char* s = (const char*)src;
    
    while (n--) {
        *d++ = *s++;
    }
    
    return dest;
}

void* memmove(void* dest, const void* src, size_t n) {
    if (!dest || !src) return dest;
    
    char* d = (char*)dest;
    const char* s = (const char*)src;
    
    if (d < s) {
        /* Copy forward */
        while (n--) {
            *d++ = *s++;
        }
    } else if (d > s) {
        /* Copy backward */
        d += n;
        s += n;
        while (n--) {
            *--d = *--s;
        }
    }
    
    return dest;
}

void* memset(void* s, int c, size_t n) {
    if (!s) return s;
    
    unsigned char* p = (unsigned char*)s;
    unsigned char value = (unsigned char)c;
    
    while (n--) {
        *p++ = value;
    }
    
    return s;
}

int memcmp(const void* s1, const void* s2, size_t n) {
    if (!s1 || !s2) {
        if (s1 == s2) return 0;
        return s1 ? 1 : -1;
    }
    
    const unsigned char* p1 = (const unsigned char*)s1;
    const unsigned char* p2 = (const unsigned char*)s2;
    
    while (n--) {
        if (*p1 != *p2) {
            return *p1 - *p2;
        }
        p1++;
        p2++;
    }
    
    return 0;
}

void* memchr(const void* s, int c, size_t n) {
    if (!s) return NULL;
    
    const unsigned char* p = (const unsigned char*)s;
    unsigned char value = (unsigned char)c;
    
    while (n--) {
        if (*p == value) {
            return (void*)p;
        }
        p++;
    }
    
    return NULL;
}

/* Error string */
char* strerror(int errnum) {
    /* Simple implementation */
    switch (errnum) {
        case 0: return "Success";
        case 1: return "Operation not permitted";
        case 2: return "No such file or directory";
        case 3: return "No such process";
        case 4: return "Interrupted system call";
        case 5: return "Input/output error";
        default: return "Unknown error";
    }
}

/* Case conversion helpers */
char* strupper(char* s) {
    if (!s) return s;
    
    char* p = s;
    while (*p) {
        if (*p >= 'a' && *p <= 'z') {
            *p = *p - 'a' + 'A';
        }
        p++;
    }
    return s;
}

char* strlower(char* s) {
    if (!s) return s;
    
    char* p = s;
    while (*p) {
        if (*p >= 'A' && *p <= 'Z') {
            *p = *p - 'A' + 'a';
        }
        p++;
    }
    return s;
}

/* Safe string functions */
size_t strlcpy(char* dest, const char* src, size_t size) {
    if (!src) return 0;
    
    size_t src_len = strlen(src);
    
    if (dest && size > 0) {
        size_t copy_len = (src_len >= size) ? size - 1 : src_len;
        memcpy(dest, src, copy_len);
        dest[copy_len] = '\0';
    }
    
    return src_len;
}

size_t strlcat(char* dest, const char* src, size_t size) {
    if (!dest || !src) return src ? strlen(src) : 0;
    
    size_t dest_len = strlen(dest);
    size_t src_len = strlen(src);
    
    if (dest_len >= size) {
        return dest_len + src_len;
    }
    
    size_t available = size - dest_len - 1;
    size_t copy_len = (src_len < available) ? src_len : available;
    
    memcpy(dest + dest_len, src, copy_len);
    dest[dest_len + copy_len] = '\0';
    
    return dest_len + src_len;
}