#include <stdio.h>
#include <string.h>

// Inline implementation for debugging
static char basename_buffer[256];

char* my_basename(char* path) {
    char* p;
    int len;
    int i;
    
    printf("  Input: '%s'\n", path ? path : "NULL");
    
    /* Handle NULL or empty string */
    if (!path || *path == '\0') {
        printf("  -> NULL/empty, returning '.'\n");
        return ".";
    }
    
    /* Make a working copy */
    len = strlen(path);
    printf("  Length: %d\n", len);
    
    if (len >= 256) {
        len = 255;
    }
    
    /* Copy character by character with bounds checking */
    for (i = 0; i < len && i < 255; i++) {
        basename_buffer[i] = path[i];
    }
    basename_buffer[i] = '\0';
    len = i;
    
    printf("  Working copy: '%s'\n", basename_buffer);
    
    /* Handle single character cases */
    if (len == 1) {
        printf("  -> Single char, returning as-is\n");
        return basename_buffer;
    }
    
    /* Remove trailing slashes */
    while (len > 1 && basename_buffer[len - 1] == '/') {
        printf("  Removing trailing slash at pos %d\n", len - 1);
        basename_buffer[len - 1] = '\0';
        len--;
    }
    
    printf("  After removing trailing slashes: '%s'\n", basename_buffer);
    
    /* If we removed everything, it was all slashes - return "/" */
    if (len == 0 || basename_buffer[0] == '\0') {
        printf("  -> All slashes removed, returning '/'\n");
        basename_buffer[0] = '/';
        basename_buffer[1] = '\0';
        return basename_buffer;
    }
    
    /* Find the last slash manually */
    p = NULL;
    for (i = 0; i < len; i++) {
        if (basename_buffer[i] == '/') {
            p = &basename_buffer[i];
        }
    }
    
    if (p) {
        printf("  Found last slash at pos %d, returning part after it\n", (int)(p - basename_buffer));
        return p + 1;
    }
    
    printf("  -> No slash found, returning whole string\n");
    return basename_buffer;
}

int main(void) {
    printf("=== Basename Debug Test ===\n");
    
    const char* test_paths[] = {
        "/usr/local/bin/program",
        "/",
        "filename",
        "./file",
        "/usr/bin/test"
    };
    
    int num_tests = sizeof(test_paths) / sizeof(test_paths[0]);
    
    for (int i = 0; i < num_tests; i++) {
        printf("\nTest %d: Testing '%s'\n", i + 1, test_paths[i]);
        
        char copy[256];
        int j;
        int len = strlen(test_paths[i]);
        
        /* Manual copy to avoid any strncpy issues */
        for (j = 0; j < len && j < 255; j++) {
            copy[j] = test_paths[i][j];
        }
        copy[j] = '\0';
        
        char* result = my_basename(copy);
        printf("  Result: '%s'\n", result);
    }
    
    printf("\n=== Debug Complete ===\n");
    return 0;
}