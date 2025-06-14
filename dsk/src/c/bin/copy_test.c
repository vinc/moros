#include <stdio.h>
#include <string.h>

int main(void) {
    printf("=== Ultra Simple Copy Test ===\n");
    
    /* Test 1: Direct string literal */
    const char* source = "hello";
    printf("Source: '%s'\n", source);
    
    /* Test 2: Manual copy to buffer */
    char buffer[100];
    int i;
    
    printf("Copying character by character:\n");
    for (i = 0; i < 5; i++) {
        buffer[i] = source[i];
        printf("  buffer[%d] = '%c'\n", i, buffer[i]);
    }
    buffer[5] = '\0';
    
    printf("Final buffer: '%s'\n", buffer);
    
    /* Test 3: Test with longer string */
    const char* source2 = "/usr/bin/test";
    char buffer2[100];
    int len = strlen(source2);
    
    printf("\nTest 2 - Source: '%s' (len=%d)\n", source2, len);
    
    for (i = 0; i < len; i++) {
        buffer2[i] = source2[i];
    }
    buffer2[len] = '\0';
    
    printf("Buffer2: '%s'\n", buffer2);
    
    /* Test 4: Test array bounds */
    printf("\nTest 3 - Array bounds check:\n");
    char buffer3[10];
    const char* source3 = "toolong";
    int len3 = strlen(source3);
    
    printf("Source3: '%s' (len=%d)\n", source3, len3);
    
    for (i = 0; i < len3 && i < 9; i++) {
        buffer3[i] = source3[i];
    }
    buffer3[i] = '\0';
    
    printf("Buffer3: '%s'\n", buffer3);
    
    printf("\n=== Copy Test Complete ===\n");
    
    return 0;
}