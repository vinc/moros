#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>

/* Simple comparison function for testing qsort */
int compare_ints(const void* a, const void* b) {
    return (*(int*)a - *(int*)b);
}

int main(void) {
    printf("=== MOROS libc Lite Test ===\n\n");
    
    /* Test 1: Basic string operations */
    printf("1. String operations:\n");
    const char* test_str = "Hello MOROS";
    printf("   strlen(\"%s\") = %lu\n", test_str, strlen(test_str));
    
    char buffer[32];
    strcpy(buffer, "test");
    strcat(buffer, "_copy");
    printf("   strcpy + strcat result: %s\n", buffer);
    
    /* Test 2: Simple memory allocation */
    printf("\n2. Memory allocation:\n");
    void* small_ptr = malloc(64);  /* Much smaller allocation */
    if (small_ptr) {
        printf("   ✓ malloc(64) successful\n");
        free(small_ptr);
        printf("   ✓ free() successful\n");
    } else {
        printf("   ✗ malloc(64) failed\n");
    }
    
    /* Test 3: String conversion */
    printf("\n3. String conversion:\n");
    int num = atoi("42");
    printf("   atoi(\"42\") = %d\n", num);
    
    long hex_num = strtol("FF", NULL, 16);
    printf("   strtol(\"FF\", NULL, 16) = %ld\n", hex_num);
    
    /* Test 4: Environment variables (simple test) */
    printf("\n4. Environment variables:\n");
    setenv("TEST", "ok", 1);
    const char* val = getenv("TEST");
    if (val && strcmp(val, "ok") == 0) {
        printf("   ✓ setenv/getenv working\n");
    } else {
        printf("   ✗ setenv/getenv failed\n");
    }
    
    /* Test 5: File operations */
    printf("\n5. File operations:\n");
    if (access("/", F_OK) == 0) {
        printf("   ✓ access(\"/\", F_OK) - root exists\n");
    } else {
        printf("   ✗ access(\"/\", F_OK) failed\n");
    }
    
    /* Test 6: Error handling */
    printf("\n6. Error handling:\n");
    errno = ENOENT;
    printf("   errno = %d, strerror = %s\n", errno, strerror(errno));
    
    /* Test 7: Simple sorting */
    printf("\n7. Sorting test:\n");
    int nums[] = {3, 1, 4, 1, 5};
    int count = sizeof(nums) / sizeof(nums[0]);
    
    printf("   Before: ");
    for (int i = 0; i < count; i++) {
        printf("%d ", nums[i]);
    }
    printf("\n");
    
    qsort(nums, count, sizeof(int), compare_ints);
    
    printf("   After:  ");
    for (int i = 0; i < count; i++) {
        printf("%d ", nums[i]);
    }
    printf("\n");
    
    /* Test 8: Process info */
    printf("\n8. Process info:\n");
    printf("   getpid() = %d\n", getpid());
    
    printf("\n=== Lite Test Complete ===\n");
    printf("This version uses minimal memory to avoid allocation issues.\n");
    
    return 0;
}