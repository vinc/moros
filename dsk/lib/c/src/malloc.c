#include <stdlib.h>
#include <stddef.h>
#include "syscall.h"

/* Memory allocation tracking structure */
struct alloc_header {
    size_t size;
    size_t align;
    unsigned int magic;
};

#define ALLOC_MAGIC 0xDEADBEEF

/* malloc implementation with allocation tracking for MOROS */
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
    
    /* Allocate space for header + requested size */
    size_t total_size = sizeof(struct alloc_header) + size;
    struct alloc_header* header = (struct alloc_header*)sys_alloc(total_size, align);
    
    if (header == NULL) {
        return NULL;
    }
    
    /* Initialize header */
    header->size = total_size;
    header->align = align;
    header->magic = ALLOC_MAGIC;
    
    /* Return pointer after header */
    return (char*)header + sizeof(struct alloc_header);
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
    
    /* Get the old allocation size */
    struct alloc_header* old_header = (struct alloc_header*)((char*)ptr - sizeof(struct alloc_header));
    if (old_header->magic != ALLOC_MAGIC) {
        /* Corruption detected */
        return NULL;
    }
    
    size_t old_user_size = old_header->size - sizeof(struct alloc_header);
    
    /* Allocate new memory */
    void* new_ptr = malloc(size);
    if (new_ptr == NULL) {
        return NULL;
    }
    
    /* Copy the smaller of old size or new size */
    size_t copy_size = (old_user_size < size) ? old_user_size : size;
    char* old_bytes = (char*)ptr;
    char* new_bytes = (char*)new_ptr;
    for (size_t i = 0; i < copy_size; i++) {
        new_bytes[i] = old_bytes[i];
    }
    
    free(ptr);
    return new_ptr;
}

void free(void* ptr) {
    if (ptr == NULL) {
        return;
    }
    
    /* Get the allocation header */
    struct alloc_header* header = (struct alloc_header*)((char*)ptr - sizeof(struct alloc_header));
    
    /* Verify magic number to detect corruption */
    if (header->magic != ALLOC_MAGIC) {
        /* Corruption detected - don't free to avoid further damage */
        return;
    }
    
    /* Free with the correct size and alignment */
    sys_free(header, header->size, header->align);
}

