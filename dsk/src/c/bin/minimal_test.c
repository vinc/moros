#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(void) {
    printf("=== Minimal Test ===\n");
    
    /* Test 1: Basic printf */
    printf("Test 1: Basic printf works\n");
    
    /* Test 2: String length */
    const char* test_str = "hello";
    size_t len = strlen(test_str);
    printf("Test 2: strlen result = %ld\n", (long)len);
    
    /* Test 3: Simple memory allocation */
    void* ptr = malloc(32);
    if (ptr) {
        printf("Test 3: malloc(32) OK\n");
        free(ptr);
        printf("Test 3: free() OK\n");
    } else {
        printf("Test 3: malloc(32) FAILED\n");
    }
    
    /* Test 4: String operations */
    char buffer[16];
    strcpy(buffer, "test");
    printf("Test 4: strcpy result = %s\n", buffer);
    
    /* Test 5: String comparison */
    int cmp = strcmp("abc", "abc");
    printf("Test 5: strcmp result = %d\n", cmp);
    
    printf("=== Minimal Test Complete ===\n");
    return 0;
}