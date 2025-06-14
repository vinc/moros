#include <stdio.h>
#include <string.h>

/* Stack-based basename function to test approach */
char* stack_basename(char* path) {
    static char result[256];
    char work[256];
    int len, i, slash_pos;
    
    /* Handle NULL or empty */
    if (!path || *path == '\0') {
        result[0] = '.';
        result[1] = '\0';
        return result;
    }
    
    /* Copy to working buffer */
    len = strlen(path);
    if (len >= 256) len = 255;
    
    for (i = 0; i < len; i++) {
        work[i] = path[i];
    }
    work[len] = '\0';
    
    printf("  Working with: '%s' (len=%d)\n", work, len);
    
    /* Handle single char */
    if (len == 1) {
        result[0] = work[0];
        result[1] = '\0';
        return result;
    }
    
    /* Remove trailing slashes */
    while (len > 1 && work[len-1] == '/') {
        work[len-1] = '\0';
        len--;
    }
    
    printf("  After removing trailing: '%s' (len=%d)\n", work, len);
    
    /* If all slashes, return "/" */
    if (len == 0 || work[0] == '\0') {
        result[0] = '/';
        result[1] = '\0';
        return result;
    }
    
    /* Find last slash */
    slash_pos = -1;
    for (i = 0; i < len; i++) {
        if (work[i] == '/') {
            slash_pos = i;
        }
    }
    
    if (slash_pos >= 0) {
        printf("  Found slash at pos %d\n", slash_pos);
        /* Copy everything after the slash */
        i = 0;
        while (work[slash_pos + 1 + i] != '\0') {
            result[i] = work[slash_pos + 1 + i];
            i++;
        }
        result[i] = '\0';
        return result;
    }
    
    /* No slash, copy whole string */
    for (i = 0; i < len; i++) {
        result[i] = work[i];
    }
    result[len] = '\0';
    return result;
}

int main(void) {
    printf("=== Stack-based Basename Test ===\n");
    
    const char* tests[] = {
        "/usr/local/bin/program",
        "/",
        "filename", 
        "./file",
        "/usr/bin/test"
    };
    
    int num_tests = sizeof(tests) / sizeof(tests[0]);
    
    for (int i = 0; i < num_tests; i++) {
        printf("\nTest %d: '%s'\n", i+1, tests[i]);
        
        char copy[256];
        int len = strlen(tests[i]);
        int j;
        
        for (j = 0; j < len; j++) {
            copy[j] = tests[i][j];
        }
        copy[len] = '\0';
        
        char* result = stack_basename(copy);
        printf("  Result: '%s'\n", result);
    }
    
    printf("\n=== Stack Test Complete ===\n");
    return 0;
}