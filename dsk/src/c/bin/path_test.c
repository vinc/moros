#include <stdio.h>
#include <string.h>
#include <libgen.h>

int main(void) {
    printf("=== Path Manipulation Test ===\n");
    
    /* Test 1: Basic path */
    printf("Test 1: Basic path\n");
    char path1[] = "/usr/local/bin/program";
    printf("  Original: %s\n", path1);
    char* dir1 = dirname(path1);
    printf("  dirname:  %s\n", dir1);
    
    /* Reset path since dirname might modify it */
    strcpy(path1, "/usr/local/bin/program");
    char* base1 = basename(path1);
    printf("  basename: %s\n", base1);
    
    /* Test 2: Root path */
    printf("\nTest 2: Root path\n");
    char path2[] = "/";
    printf("  Original: %s\n", path2);
    char* dir2 = dirname(path2);
    printf("  dirname:  %s\n", dir2);
    
    strcpy(path2, "/");
    char* base2 = basename(path2);
    printf("  basename: %s\n", base2);
    
    /* Test 3: No path separators */
    printf("\nTest 3: No path separators\n");
    char path3[] = "filename";
    printf("  Original: %s\n", path3);
    char* dir3 = dirname(path3);
    printf("  dirname:  %s\n", dir3);
    
    strcpy(path3, "filename");
    char* base3 = basename(path3);
    printf("  basename: %s\n", base3);
    
    /* Test 4: Current directory */
    printf("\nTest 4: Current directory\n");
    char path4[] = "./file";
    printf("  Original: %s\n", path4);
    char* dir4 = dirname(path4);
    printf("  dirname:  %s\n", dir4);
    
    strcpy(path4, "./file");
    char* base4 = basename(path4);
    printf("  basename: %s\n", base4);
    
    /* Test 5: Check if functions return valid pointers */
    printf("\nTest 5: Pointer validation\n");
    char path5[] = "/test/path";
    char* dir5 = dirname(path5);
    char* base5 = basename(path5);
    
    printf("  dirname pointer: %p\n", (void*)dir5);
    printf("  basename pointer: %p\n", (void*)base5);
    printf("  dirname string: '%s'\n", dir5 ? dir5 : "NULL");
    printf("  basename string: '%s'\n", base5 ? base5 : "NULL");
    
    /* Test 6: Manual string operations to verify strchr works */
    printf("\nTest 6: Manual string checks\n");
    char test_str[] = "/usr/bin/test";
    char* last_slash = strrchr(test_str, '/');
    printf("  String: %s\n", test_str);
    printf("  Last slash at: %p\n", (void*)last_slash);
    if (last_slash) {
        printf("  After last slash: '%s'\n", last_slash + 1);
        printf("  Before last slash length: %ld\n", (long)(last_slash - test_str));
    }
    
    printf("\n=== Path Test Complete ===\n");
    return 0;
}