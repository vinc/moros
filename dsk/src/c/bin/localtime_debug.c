#include <stdio.h>
#include <time.h>
#include <string.h>

int main(void) {
    printf("=== localtime_r Buffer Corruption Debug ===\n");
    
    time_t test_time = 946684801; /* Jan 1, 2000 00:00:01 UTC */
    printf("Testing timestamp: %ld\n", (long)test_time);
    
    /* Test 1: Create buffer and initialize with known values */
    struct tm my_tm;
    printf("\nInitializing buffer with test values...\n");
    my_tm.tm_sec = 99;
    my_tm.tm_min = 88;
    my_tm.tm_hour = 77;
    my_tm.tm_mday = 66;
    my_tm.tm_mon = 55;
    my_tm.tm_year = 44;
    my_tm.tm_wday = 33;
    my_tm.tm_yday = 22;
    my_tm.tm_isdst = 11;
    
    printf("Buffer initialized - checking values:\n");
    printf("  tm_sec = %d (should be 99)\n", my_tm.tm_sec);
    printf("  tm_wday = %d (should be 33)\n", my_tm.tm_wday);
    printf("  tm_year = %d (should be 44)\n", my_tm.tm_year);
    
    /* Test 2: Call localtime_r and check immediately */
    printf("\nCalling localtime_r...\n");
    printf("Buffer address: %p\n", (void*)&my_tm);
    
    struct tm* result = localtime_r(&test_time, &my_tm);
    
    printf("localtime_r returned: %p\n", (void*)result);
    printf("Buffer address: %p\n", (void*)&my_tm);
    
    if (result != &my_tm) {
        printf("ERROR: Returned pointer doesn't match our buffer!\n");
    }
    
    /* Test 3: Check values immediately after call */
    printf("\nImmediate check after localtime_r:\n");
    printf("  tm_sec = %d (should be 1)\n", my_tm.tm_sec);
    printf("  tm_min = %d (should be 0)\n", my_tm.tm_min);
    printf("  tm_hour = %d (should be 0)\n", my_tm.tm_hour);
    printf("  tm_mday = %d (should be 1)\n", my_tm.tm_mday);
    printf("  tm_mon = %d (should be 0)\n", my_tm.tm_mon);
    printf("  tm_year = %d (should be 100)\n", my_tm.tm_year);
    printf("  tm_wday = %d (should be 6)\n", my_tm.tm_wday);
    
    /* Test 4: Check if any values are correct */
    int correct_count = 0;
    if (my_tm.tm_sec == 1) correct_count++;
    if (my_tm.tm_min == 0) correct_count++;
    if (my_tm.tm_hour == 0) correct_count++;
    if (my_tm.tm_mday == 1) correct_count++;
    if (my_tm.tm_mon == 0) correct_count++;
    if (my_tm.tm_year == 100) correct_count++;
    if (my_tm.tm_wday == 6) correct_count++;
    
    printf("\nCorrect values: %d out of 7\n", correct_count);
    
    /* Test 5: Check via returned pointer */
    printf("\nChecking via returned pointer:\n");
    if (result) {
        printf("  result->tm_sec = %d (should be 1)\n", result->tm_sec);
        printf("  result->tm_wday = %d (should be 6)\n", result->tm_wday);
        printf("  result->tm_year = %d (should be 100)\n", result->tm_year);
    }
    
    /* Test 6: Test multiple calls */
    printf("\nTesting multiple localtime_r calls:\n");
    
    struct tm tm1, tm2, tm3;
    time_t times[] = {0, 946684800, 946684801};
    struct tm* buffers[] = {&tm1, &tm2, &tm3};
    char* names[] = {"Epoch", "Y2K", "Y2K+1"};
    
    for (int i = 0; i < 3; i++) {
        struct tm* res = localtime_r(&times[i], buffers[i]);
        printf("%s: tm_year=%d, tm_wday=%d (ptr=%p)\n", 
               names[i], 
               res ? res->tm_year : -1,
               res ? res->tm_wday : -1,
               (void*)res);
    }
    
    /* Test 7: Simple manual assignment test */
    printf("\nManual assignment test:\n");
    struct tm manual;
    manual.tm_year = 100;
    manual.tm_wday = 6;
    printf("Set tm_year=100, tm_wday=6\n");
    printf("Read back: tm_year=%d, tm_wday=%d\n", manual.tm_year, manual.tm_wday);
    
    printf("\n=== Test Complete ===\n");
    return 0;
}