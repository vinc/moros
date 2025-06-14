#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <dirent.h>
#include <errno.h>
#include <time.h>
#include <libgen.h>

/* Comparison function for qsort */
int compare_ints(const void* a, const void* b) {
    return (*(int*)a - *(int*)b);
}

int main(int argc, char* argv[]) {
    printf("=== MOROS libc Extended Functions Test ===\n\n");
    
    /* Test 1: Basic file operations */
    printf("1. Testing file operations:\n");
    
    if (access("/ini", F_OK) == 0) {
        printf("   ✓ access() - /ini exists\n");
    } else {
        printf("   ✗ access() - /ini not found (errno: %d)\n", errno);
    }
    
    /* Test 2: Environment variables */
    printf("\n2. Testing environment variables:\n");
    
    setenv("TEST_VAR", "hello_moros", 1);
    const char* test_val = getenv("TEST_VAR");
    if (test_val && strcmp(test_val, "hello_moros") == 0) {
        printf("   ✓ setenv/getenv - TEST_VAR = %s\n", test_val);
    } else {
        printf("   ✗ setenv/getenv failed\n");
    }
    
    /* Test 3: String conversion */
    printf("\n3. Testing string conversion:\n");
    
    const char* num_str = "12345";
    int num = atoi(num_str);
    printf("   atoi(\"%s\") = %d\n", num_str, num);
    
    char* endptr;
    long hex_num = strtol("0x1A2B", &endptr, 16);
    printf("   strtol(\"0x1A2B\", &endptr, 16) = %ld\n", hex_num);
    
    /* Test 4: Path manipulation */
    printf("\n4. Testing path manipulation:\n");
    
    char test_path[] = "/usr/local/bin/program";
    char* dir = dirname(test_path);
    printf("   dirname(\"%s\") = %s\n", "/usr/local/bin/program", dir);
    
    char test_path2[] = "/usr/local/bin/program";
    char* base = basename(test_path2);
    printf("   basename(\"%s\") = %s\n", "/usr/local/bin/program", base);
    
    /* Test 5: File stats */
    printf("\n5. Testing file statistics:\n");
    
    struct stat st;
    if (stat("/", &st) == 0) {
        printf("   ✓ stat(\"/\") successful\n");
        printf("     Size: %ld bytes\n", st.st_size);
        printf("     Mode: 0%x\n", st.st_mode);
        if (S_ISDIR(st.st_mode)) {
            printf("     Type: Directory\n");
        } else if (S_ISREG(st.st_mode)) {
            printf("     Type: Regular file\n");
        } else {
            printf("     Type: Other\n");
        }
    } else {
        printf("   ✗ stat(\"/\") failed (errno: %d)\n", errno);
    }
    
    /* Test 6: Directory operations */
    printf("\n6. Testing directory operations:\n");
    
    DIR* dir_handle = opendir("/");
    if (dir_handle) {
        printf("   ✓ opendir(\"/\") successful\n");
        
        struct dirent* entry;
        int count = 0;
        while ((entry = readdir(dir_handle)) != NULL && count < 5) {
            printf("     Entry: %s (type: %d)\n", entry->d_name, entry->d_type);
            count++;
        }
        
        if (count == 0) {
            printf("     (Directory appears empty - readdir needs kernel support)\n");
        }
        
        closedir(dir_handle);
        printf("   ✓ closedir() successful\n");
    } else {
        printf("   ✗ opendir(\"/\") failed (errno: %d)\n", errno);
    }
    
    /* Test 7: Time functions */
    printf("\n7. Testing time functions:\n");
    
    time_t current_time = time(NULL);
    printf("   time(NULL) = %ld\n", current_time);
    
    struct tm* tm_info = localtime(&current_time);
    if (tm_info) {
        printf("   localtime: %ld-%ld-%ld %ld:%ld:%ld\n",
               (long)(tm_info->tm_year + 1900), (long)(tm_info->tm_mon + 1), (long)tm_info->tm_mday,
               (long)tm_info->tm_hour, (long)tm_info->tm_min, (long)tm_info->tm_sec);
        printf("   tm_info ptr: %p, wday: %d, mon: %d\n", (void*)tm_info, tm_info->tm_wday, tm_info->tm_mon);
    } else {
        printf("   localtime returned NULL!\n");
    }
    
    char* time_str = ctime(&current_time);
    if (time_str) {
        printf("   ctime: '%s'", time_str); /* ctime includes newline */
        printf("   ctime length: %d\n", (int)strlen(time_str));
    } else {
        printf("   ctime returned NULL!\n");
    }
    
    /* Test 8: Memory allocation */
    printf("\n8. Testing memory allocation:\n");
    
    void* ptr1 = malloc(1024);
    if (ptr1) {
        printf("   ✓ malloc(1024) successful\n");
        free(ptr1);
        printf("   ✓ free() successful\n");
    } else {
        printf("   ✗ malloc(1024) failed\n");
    }
    
    void* ptr2 = calloc(10, sizeof(int));
    if (ptr2) {
        printf("   ✓ calloc(10, sizeof(int)) successful\n");
        free(ptr2);
    } else {
        printf("   ✗ calloc() failed\n");
    }
    
    /* Test 9: Error handling */
    printf("\n9. Testing error handling:\n");
    
    errno = ENOENT;
    printf("   errno = %d (%s)\n", errno, strerror(errno));
    
    errno = ENOMEM;
    printf("   errno = %d (%s)\n", errno, strerror(errno));
    
    /* Test 10: Sorting */
    printf("\n10. Testing sorting:\n");
    
    int numbers[] = {5, 2, 8, 1, 9, 3};
    int count_nums = sizeof(numbers) / sizeof(numbers[0]);
    
    printf("    Before sort: ");
    for (int i = 0; i < count_nums; i++) {
        printf("%d ", numbers[i]);
    }
    printf("\n");
    
    qsort(numbers, count_nums, sizeof(int), compare_ints);
    
    printf("    After sort:  ");
    for (int i = 0; i < count_nums; i++) {
        printf("%d ", numbers[i]);
    }
    printf("\n");
    
    /* Test 11: System command (if available) */
    printf("\n11. Testing system command:\n");
    printf("    system() availability: %s\n", 
           system(NULL) ? "Available" : "Not available");
    
    printf("\n=== Test Complete ===\n");
    printf("If most tests show ✓, the libc extensions are working!\n");
    printf("Some functions may show limited functionality due to missing kernel support.\n");
    
    return 0;
}