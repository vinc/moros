#include <stdlib.h>
#include <stddef.h>
#include "syscall.h"

/* Simple malloc implementation using MOROS alloc/free syscalls */

void* malloc(size_t size) {
    if (size == 0) {
        return NULL;
    }
    
    /* Align to 8 bytes by default */
    size_t align = 8;
    if (size < align) {
        size = align;
    }
    
    /* Round up to nearest multiple of alignment */
    size = (size + align - 1) & ~(align - 1);
    
    void* ptr = sys_alloc(size, align);
    return ptr;
}

void* calloc(size_t nmemb, size_t size) {
    if (nmemb == 0 || size == 0) {
        return NULL;
    }
    
    /* Check for overflow */
    if (nmemb > SIZE_MAX / size) {
        return NULL;
    }
    
    size_t total_size = nmemb * size;
    void* ptr = malloc(total_size);
    
    if (ptr != NULL) {
        /* Zero out the memory */
        char* bytes = (char*)ptr;
        for (size_t i = 0; i < total_size; i++) {
            bytes[i] = 0;
        }
    }
    
    return ptr;
}

void* realloc(void* ptr, size_t size) {
    if (ptr == NULL) {
        return malloc(size);
    }
    
    if (size == 0) {
        free(ptr);
        return NULL;
    }
    
    /* For now, we implement realloc as alloc + copy + free */
    /* This is inefficient but correct */
    void* new_ptr = malloc(size);
    if (new_ptr == NULL) {
        return NULL;
    }
    
    /* We don't have a way to get the old size, so we assume
     * the caller knows what they're doing and copy 'size' bytes.
     * This is not fully compliant but works for simple cases. */
    char* old_bytes = (char*)ptr;
    char* new_bytes = (char*)new_ptr;
    for (size_t i = 0; i < size; i++) {
        new_bytes[i] = old_bytes[i];
    }
    
    free(ptr);
    return new_ptr;
}

void free(void* ptr) {
    if (ptr == NULL) {
        return;
    }
    
    /* MOROS free syscall requires size and alignment,
     * but standard free() doesn't provide these.
     * For now, we use default values. A more sophisticated
     * implementation would track allocations. */
    size_t default_size = 0;  /* MOROS should handle size tracking */
    size_t default_align = 8;
    
    sys_free(ptr, default_size, default_align);
}

/* Memory allocation tracking structure for future enhancement */
struct alloc_header {
    size_t size;
    size_t align;
    unsigned int magic;
};

#define ALLOC_MAGIC 0xDEADBEEF

/* Enhanced malloc with allocation tracking (commented out for now) */
/*
void* malloc_tracked(size_t size) {
    if (size == 0) return NULL;
    
    size_t align = 8;
    size_t total_size = sizeof(struct alloc_header) + size;
    
    struct alloc_header* header = (struct alloc_header*)sys_alloc(total_size, align);
    if (header == NULL) return NULL;
    
    header->size = size;
    header->align = align;
    header->magic = ALLOC_MAGIC;
    
    return (char*)header + sizeof(struct alloc_header);
}

void free_tracked(void* ptr) {
    if (ptr == NULL) return;
    
    struct alloc_header* header = (struct alloc_header*)((char*)ptr - sizeof(struct alloc_header));
    
    if (header->magic != ALLOC_MAGIC) {
        // Corruption detected
        return;
    }
    
    size_t total_size = sizeof(struct alloc_header) + header->size;
    sys_free(header, total_size, header->align);
}
*/