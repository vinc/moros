#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <dirent.h>
#include <errno.h>

int main(void) {
    printf("=== MOROS libc Debug Test ===\n");
    printf("Testing each function individually to find memory issue...\n\n");
    
    /* Step 1: Basic operations */
    printf("Step 1: Basic string operations\n");
    const char* test = "hello";
    printf("  strlen(\"%s\") = %lu\n", test, strlen(test));
    printf("  Step 1 complete\n\n");
    
    /* Step 2: Small memory allocation */
    printf("Step 2: Small memory allocation\n");
    void* ptr1 = malloc(32);
    if (ptr1) {
        printf("  ✓ malloc(32) OK\n");
        free(ptr1);
        printf("  ✓ free() OK\n");
    } else {
        printf("  ✗ malloc(32) FAILED\n");
        return 1;
    }
    printf("  Step 2 complete\n\n");
    
    /* Step 3: Medium memory allocation */
    printf("Step 3: Medium memory allocation\n");
    void* ptr2 = malloc(128);
    if (ptr2) {
        printf("  ✓ malloc(128) OK\n");
        free(ptr2);
        printf("  ✓ free() OK\n");
    } else {
        printf("  ✗ malloc(128) FAILED\n");
        return 1;
    }
    printf("  Step 3 complete\n\n");
    
    /* Step 4: Large memory allocation */
    printf("Step 4: Large memory allocation\n");
    void* ptr3 = malloc(1024);
    if (ptr3) {
        printf("  ✓ malloc(1024) OK\n");
        free(ptr3);
        printf("  ✓ free() OK\n");
    } else {
        printf("  ✗ malloc(1024) FAILED - this might be the issue\n");
        return 1;
    }
    printf("  Step 4 complete\n\n");
    
    /* Step 5: Environment variables */
    printf("Step 5: Environment variables\n");
    int env_result = setenv("DEBUG_TEST", "value", 1);
    if (env_result == 0) {
        printf("  ✓ setenv() OK\n");
        const char* val = getenv("DEBUG_TEST");
        if (val) {
            printf("  ✓ getenv() = %s\n", val);
        }
    } else {
        printf("  ✗ setenv() FAILED\n");
    }
    printf("  Step 5 complete\n\n");
    
    /* Step 6: File access */
    printf("Step 6: File access\n");
    if (access("/", F_OK) == 0) {
        printf("  ✓ access(\"/\") OK\n");
    } else {
        printf("  ✗ access(\"/\") FAILED\n");
    }
    printf("  Step 6 complete\n\n");
    
    /* Step 7: File stat */
    printf("Step 7: File stat\n");
    struct stat st;
    if (stat("/", &st) == 0) {
        printf("  ✓ stat(\"/\") OK\n");
    } else {
        printf("  ✗ stat(\"/\") FAILED\n");
    }
    printf("  Step 7 complete\n\n");
    
    /* Step 8: Directory operations - this might be the culprit */
    printf("Step 8: Directory operations (4KB allocation)\n");
    DIR* dir = opendir("/");
    if (dir) {
        printf("  ✓ opendir(\"/\") OK\n");
        closedir(dir);
        printf("  ✓ closedir() OK\n");
    } else {
        printf("  ✗ opendir(\"/\") FAILED - 4KB allocation issue?\n");
    }
    printf("  Step 8 complete\n\n");
    
    /* Step 9: Multiple allocations */
    printf("Step 9: Multiple small allocations\n");
    void* ptrs[10];
    int allocated = 0;
    
    for (int i = 0; i < 10; i++) {
        ptrs[i] = malloc(64);
        if (ptrs[i]) {
            allocated++;
        } else {
            printf("  ✗ malloc(%d) failed at iteration %d\n", 64, i);
            break;
        }
    }
    
    printf("  Successfully allocated %d/10 blocks\n", allocated);
    
    /* Free all allocated blocks */
    for (int i = 0; i < allocated; i++) {
        free(ptrs[i]);
    }
    printf("  Freed all blocks\n");
    printf("  Step 9 complete\n\n");
    
    printf("=== Debug Test Complete ===\n");
    printf("If this runs completely, the issue is likely in the full test's\n");
    printf("combination of operations or a specific large allocation.\n");
    
    return 0;
}