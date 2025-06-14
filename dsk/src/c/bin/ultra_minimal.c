#include <stdio.h>
#include <string.h>

int main(void) {
    /* Test 1: Direct printf without any variables */
    printf("Test 1: Hello World\n");
    
    /* Test 2: Simple string length */
    printf("Test 2: strlen test\n");
    const char* str = "hello";
    size_t len = strlen(str);
    
    /* Test 3: Manual printf of length to avoid format issues */
    printf("Test 3: Length is ");
    if (len == 5) {
        printf("5 (correct)\n");
    } else {
        printf("wrong\n");
    }
    
    /* Test 4: Simple string copy */
    printf("Test 4: strcpy test\n");
    char dest[10];
    strcpy(dest, "test");
    printf("Copied: ");
    printf(dest);
    printf("\n");
    
    /* Test 5: Simple comparison */
    printf("Test 5: strcmp test\n");
    int result = strcmp("abc", "abc");
    if (result == 0) {
        printf("Strings match (correct)\n");
    } else {
        printf("Strings don't match (wrong)\n");
    }
    
    printf("Ultra minimal test complete - no syscalls used\n");
    return 0;
}