#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>

int main(void) {
    printf("=== Panic Isolation Test ===\n");
    printf("Testing each operation that might cause the panic...\n\n");
    
    /* Test 1: Basic string operations (we know this works) */
    printf("Test 1: Basic string ops\n");
    const char* test_str = "hello";
    printf("  strlen result: %ld\n", (long)strlen(test_str));
    printf("  Test 1 OK\n\n");
    
    /* Test 2: String conversion (might be the issue) */
    printf("Test 2: String conversion\n");
    int num = atoi("42");
    printf("  atoi result: %d\n", num);
    
    printf("  About to call strtol...\n");
    long hex_num = strtol("FF", NULL, 16);
    printf("  strtol result: %ld\n", hex_num);
    printf("  Test 2 OK\n\n");
    
    /* Test 3: Environment variables - step by step */
    printf("Test 3: Environment variables\n");
    printf("  About to call setenv...\n");
    int setenv_result = setenv("TEST", "ok", 1);
    printf("  setenv returned: %d\n", setenv_result);
    
    printf("  About to call getenv...\n");
    const char* val = getenv("TEST");
    if (val) {
        printf("  getenv returned: %s\n", val);
        
        printf("  About to call strcmp...\n");
        int cmp = strcmp(val, "ok");
        printf("  strcmp returned: %d\n", cmp);
        
        if (cmp == 0) {
            printf("  ✓ Environment variables working\n");
        } else {
            printf("  ✗ Environment variables comparison failed\n");
        }
    } else {
        printf("  ✗ getenv returned NULL\n");
    }
    printf("  Test 3 OK\n\n");
    
    /* Test 4: File access - this is where debug_test showed failure */
    printf("Test 4: File access\n");
    printf("  About to call access...\n");
    
    /* Check errno before the call */
    errno = 0;
    
    int access_result = access("/", F_OK);
    printf("  access returned: %d\n", access_result);
    printf("  errno after access: %d\n", errno);
    
    if (access_result == 0) {
        printf("  ✓ access OK\n");
    } else {
        printf("  ✗ access failed (expected)\n");
    }
    printf("  Test 4 OK\n\n");
    
    /* Test 5: Error handling */
    printf("Test 5: Error handling\n");
    printf("  About to call strerror...\n");
    const char* err_str = strerror(ENOENT);
    printf("  strerror result: %s\n", err_str);
    printf("  Test 5 OK\n\n");
    
    printf("=== All tests completed successfully ===\n");
    printf("If we reach this point, the panic is not in these operations.\n");
    
    return 0;
}