#include <stdio.h>
#include <string.h>

int main(void) {
    printf("=== String Function Debug Test ===\n");
    
    /* Test 1: Basic string length */
    const char* test1 = "hello";
    size_t len1 = strlen(test1);
    printf("Test 1: strlen('%s') = %d\n", test1, (int)len1);
    
    /* Test 2: String copy */
    char buffer[256];
    const char* test2 = "/usr/bin/test";
    strcpy(buffer, test2);
    printf("Test 2: strcpy('%s') -> '%s'\n", test2, buffer);
    
    /* Test 3: String length of copied string */
    size_t len2 = strlen(buffer);
    printf("Test 3: strlen(buffer) = %d\n", (int)len2);
    
    /* Test 4: strncpy test */
    char buffer2[256];
    const char* test3 = "filename";
    size_t len3 = strlen(test3);
    strncpy(buffer2, test3, len3);
    buffer2[len3] = '\0';
    printf("Test 4: strncpy('%s') -> '%s' (len=%d)\n", test3, buffer2, (int)len3);
    
    /* Test 5: Character array access */
    const char* test4 = "/path/to/file";
    printf("Test 5: String '%s'\n", test4);
    for (int i = 0; i < 13 && test4[i]; i++) {
        printf("  [%d] = '%c' (0x%02x)\n", i, test4[i], (unsigned char)test4[i]);
    }
    
    /* Test 6: strrchr test */
    const char* test5 = "/usr/local/bin/program";
    char* last_slash = strrchr(test5, '/');
    if (last_slash) {
        printf("Test 6: strrchr found slash at position %d\n", (int)(last_slash - test5));
        printf("  After slash: '%s'\n", last_slash + 1);
    } else {
        printf("Test 6: strrchr did not find slash\n");
    }
    
    /* Test 7: Manual basename implementation */
    printf("\nTest 7: Manual basename of '%s'\n", test5);
    char manual_buffer[256];
    size_t manual_len = strlen(test5);
    printf("  Length: %d\n", (int)manual_len);
    
    /* Copy manually character by character */
    for (size_t i = 0; i < manual_len && i < 255; i++) {
        manual_buffer[i] = test5[i];
    }
    manual_buffer[manual_len] = '\0';
    printf("  Manual copy: '%s'\n", manual_buffer);
    
    /* Find last slash manually */
    char* manual_slash = NULL;
    for (size_t i = 0; i < manual_len; i++) {
        if (manual_buffer[i] == '/') {
            manual_slash = &manual_buffer[i];
        }
    }
    
    if (manual_slash) {
        printf("  Manual slash at position %d\n", (int)(manual_slash - manual_buffer));
        printf("  Manual basename: '%s'\n", manual_slash + 1);
    } else {
        printf("  No slash found manually\n");
    }
    
    printf("\n=== String Debug Complete ===\n");
    return 0;
}