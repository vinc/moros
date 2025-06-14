#include <stdio.h>
#include <string.h>
#include <libgen.h>

int main(void) {
    printf("=== Dirname/Basename Debug Test ===\n");
    
    const char* test_paths[] = {
        "/usr/local/bin/program",
        "/",
        "filename",
        "./file",
        "/usr/bin/test",
        "/ini/test",
        "/dev/clk/epoch",
        ""
    };
    
    int num_tests = sizeof(test_paths) / sizeof(test_paths[0]);
    
    for (int i = 0; i < num_tests; i++) {
        printf("\nTest %d: '%s'\n", i + 1, test_paths[i]);
        
        /* Make copies since dirname/basename may modify the string */
        char copy1[256], copy2[256];
        strncpy(copy1, test_paths[i], sizeof(copy1) - 1);
        copy1[sizeof(copy1) - 1] = '\0';
        strncpy(copy2, test_paths[i], sizeof(copy2) - 1);
        copy2[sizeof(copy2) - 1] = '\0';
        
        char* dir = dirname(copy1);
        char* base = basename(copy2);
        
        printf("  dirname:  '%s' (ptr: %p)\n", dir ? dir : "NULL", (void*)dir);
        printf("  basename: '%s' (ptr: %p)\n", base ? base : "NULL", (void*)base);
        
        /* Check if results are empty strings */
        if (dir && strlen(dir) == 0) {
            printf("  WARNING: dirname returned empty string!\n");
        }
        if (base && strlen(base) == 0) {
            printf("  WARNING: basename returned empty string!\n");
        }
    }
    
    printf("\n=== Debug Test Complete ===\n");
    return 0;
}