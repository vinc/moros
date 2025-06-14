#include <stdio.h>
#include <time.h>
#include <string.h>

/* Test struct to see if the issue is specific to tm */
struct test_struct {
    int a;
    int b;
    int c;
    int d;
    int e;
    int f;
    int g;
    int h;
    int i;
};

/* Global test variable */
struct test_struct global_test = {0};

/* Global tm variable */
struct tm global_tm = {0};

int main(void) {
    printf("=== Memory Debug Test ===\n");
    
    /* Test 1: Can we set and read a simple global struct? */
    printf("\nTest 1: Simple global struct\n");
    global_test.a = 123;
    global_test.b = 456;
    global_test.c = 789;
    printf("Set: a=123, b=456, c=789\n");
    printf("Read: a=%d, b=%d, c=%d\n", global_test.a, global_test.b, global_test.c);
    
    if (global_test.a == 123 && global_test.b == 456 && global_test.c == 789) {
        printf("✓ Simple global struct works!\n");
    } else {
        printf("✗ Simple global struct FAILED!\n");
    }
    
    /* Test 2: Can we set and read the global tm struct directly? */
    printf("\nTest 2: Global tm struct direct access\n");
    global_tm.tm_year = 100;
    global_tm.tm_mon = 0;
    global_tm.tm_mday = 1;
    global_tm.tm_hour = 0;
    global_tm.tm_min = 0;
    global_tm.tm_sec = 1;
    global_tm.tm_wday = 6;
    
    printf("Set: year=100, mon=0, mday=1, hour=0, min=0, sec=1, wday=6\n");
    printf("Read: year=%d, mon=%d, mday=%d, hour=%d, min=%d, sec=%d, wday=%d\n",
           global_tm.tm_year, global_tm.tm_mon, global_tm.tm_mday,
           global_tm.tm_hour, global_tm.tm_min, global_tm.tm_sec, global_tm.tm_wday);
    
    if (global_tm.tm_year == 100 && global_tm.tm_mon == 0 && global_tm.tm_mday == 1 &&
        global_tm.tm_hour == 0 && global_tm.tm_min == 0 && global_tm.tm_sec == 1 && global_tm.tm_wday == 6) {
        printf("✓ Global tm struct works!\n");
    } else {
        printf("✗ Global tm struct FAILED!\n");
    }
    
    /* Test 3: Can we return a pointer to the global tm struct? */
    printf("\nTest 3: Pointer to global tm struct\n");
    struct tm* ptr = &global_tm;
    printf("Pointer access: year=%d, mon=%d, mday=%d, hour=%d, min=%d, sec=%d, wday=%d\n",
           ptr->tm_year, ptr->tm_mon, ptr->tm_mday,
           ptr->tm_hour, ptr->tm_min, ptr->tm_sec, ptr->tm_wday);
    
    if (ptr->tm_year == 100 && ptr->tm_mon == 0 && ptr->tm_mday == 1 &&
        ptr->tm_hour == 0 && ptr->tm_min == 0 && ptr->tm_sec == 1 && ptr->tm_wday == 6) {
        printf("✓ Pointer to global tm struct works!\n");
    } else {
        printf("✗ Pointer to global tm struct FAILED!\n");
    }
    
    /* Test 4: Test what happens when we call gmtime */
    printf("\nTest 4: Call gmtime and check immediately\n");
    time_t test_time = 946684801;
    printf("Calling gmtime(%ld)...\n", (long)test_time);
    
    struct tm* result = gmtime(&test_time);
    if (result == NULL) {
        printf("✗ gmtime returned NULL!\n");
    } else {
        printf("gmtime returned: %p\n", (void*)result);
        printf("Global tm address: %p\n", (void*)&global_tm);
        
        printf("Immediate read: year=%d, mon=%d, mday=%d, hour=%d, min=%d, sec=%d, wday=%d\n",
               result->tm_year, result->tm_mon, result->tm_mday,
               result->tm_hour, result->tm_min, result->tm_sec, result->tm_wday);
        
        /* Check if our global tm was affected */
        printf("Global tm after gmtime: year=%d, mon=%d, mday=%d, hour=%d, min=%d, sec=%d, wday=%d\n",
               global_tm.tm_year, global_tm.tm_mon, global_tm.tm_mday,
               global_tm.tm_hour, global_tm.tm_min, global_tm.tm_sec, global_tm.tm_wday);
    }
    
    /* Test 5: Check memory addresses */
    printf("\nTest 5: Memory addresses\n");
    printf("Address of global_test: %p\n", (void*)&global_test);
    printf("Address of global_tm: %p\n", (void*)&global_tm);
    printf("Address of test_time: %p\n", (void*)&test_time);
    
    /* Test 6: Try to manually create a tm struct on stack */
    printf("\nTest 6: Stack-based tm struct\n");
    struct tm stack_tm;
    stack_tm.tm_year = 100;
    stack_tm.tm_mon = 0;
    stack_tm.tm_mday = 1;
    stack_tm.tm_hour = 0;
    stack_tm.tm_min = 0;
    stack_tm.tm_sec = 1;
    stack_tm.tm_wday = 6;
    stack_tm.tm_yday = 0;
    stack_tm.tm_isdst = 0;
    
    printf("Stack tm: year=%d, mon=%d, mday=%d, hour=%d, min=%d, sec=%d, wday=%d\n",
           stack_tm.tm_year, stack_tm.tm_mon, stack_tm.tm_mday,
           stack_tm.tm_hour, stack_tm.tm_min, stack_tm.tm_sec, stack_tm.tm_wday);
    
    if (stack_tm.tm_year == 100 && stack_tm.tm_mon == 0 && stack_tm.tm_mday == 1 &&
        stack_tm.tm_hour == 0 && stack_tm.tm_min == 0 && stack_tm.tm_sec == 1 && stack_tm.tm_wday == 6) {
        printf("✓ Stack tm struct works!\n");
    } else {
        printf("✗ Stack tm struct FAILED!\n");
    }
    
    printf("\n=== Memory Debug Complete ===\n");
    return 0;
}