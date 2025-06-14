#include <stdio.h>
#include <time.h>

int main(void) {
    printf("=== gmtime_r Test (Caller Provides Buffer) ===\n");
    
    /* Test the specific timestamp that was failing */
    time_t test_time = 946684801; /* Jan 1, 2000 00:00:01 UTC */
    printf("Testing timestamp: %ld\n", (long)test_time);
    printf("Expected: Sat Jan 01 00:00:01 2000\n\n");
    
    /* Create our own buffer on the stack */
    struct tm my_tm;
    printf("Our buffer address: %p\n", (void*)&my_tm);
    
    /* Call gmtime_r with our buffer */
    struct tm* result = gmtime_r(&test_time, &my_tm);
    
    if (result == NULL) {
        printf("ERROR: gmtime_r returned NULL!\n");
        return 1;
    }
    
    printf("gmtime_r returned: %p\n", (void*)result);
    printf("Our buffer address: %p\n", (void*)&my_tm);
    
    if (result != &my_tm) {
        printf("WARNING: Returned pointer doesn't match our buffer!\n");
    }
    
    printf("\nReading from returned pointer:\n");
    printf("  Year: %d (should be 2000)\n", result->tm_year + 1900);
    printf("  Month: %d (should be 1)\n", result->tm_mon + 1);
    printf("  Day: %d (should be 1)\n", result->tm_mday);
    printf("  Hour: %d (should be 0)\n", result->tm_hour);
    printf("  Min: %d (should be 0)\n", result->tm_min);
    printf("  Sec: %d (should be 1)\n", result->tm_sec);
    printf("  Weekday: %d (should be 6 for Saturday)\n", result->tm_wday);
    
    printf("\nReading directly from our buffer:\n");
    printf("  Year: %d (should be 2000)\n", my_tm.tm_year + 1900);
    printf("  Month: %d (should be 1)\n", my_tm.tm_mon + 1);
    printf("  Day: %d (should be 1)\n", my_tm.tm_mday);
    printf("  Hour: %d (should be 0)\n", my_tm.tm_hour);
    printf("  Min: %d (should be 0)\n", my_tm.tm_min);
    printf("  Sec: %d (should be 1)\n", my_tm.tm_sec);
    printf("  Weekday: %d (should be 6 for Saturday)\n", my_tm.tm_wday);
    
    /* Check if values are correct */
    if (my_tm.tm_year == 100 && my_tm.tm_mon == 0 && my_tm.tm_mday == 1 &&
        my_tm.tm_hour == 0 && my_tm.tm_min == 0 && my_tm.tm_sec == 1 && my_tm.tm_wday == 6) {
        printf("\n✓ gmtime_r SUCCESS! All values are correct!\n");
    } else {
        printf("\n✗ gmtime_r FAILED! Some values are incorrect.\n");
    }
    
    /* Test localtime_r as well */
    printf("\n=== Testing localtime_r ===\n");
    struct tm local_tm;
    result = localtime_r(&test_time, &local_tm);
    
    if (result) {
        printf("localtime_r: Year %d, Month %d, Day %d, %02d:%02d:%02d\n",
               local_tm.tm_year + 1900, local_tm.tm_mon + 1, local_tm.tm_mday,
               local_tm.tm_hour, local_tm.tm_min, local_tm.tm_sec);
        
        if (local_tm.tm_year == 100 && local_tm.tm_mon == 0 && local_tm.tm_mday == 1 &&
            local_tm.tm_hour == 0 && local_tm.tm_min == 0 && local_tm.tm_sec == 1) {
            printf("✓ localtime_r also works!\n");
        } else {
            printf("✗ localtime_r failed\n");
        }
    } else {
        printf("✗ localtime_r returned NULL\n");
    }
    
    /* Test Y2K as well */
    printf("\n=== Testing Y2K (946684800) ===\n");
    time_t y2k = 946684800;
    struct tm y2k_tm;
    result = gmtime_r(&y2k, &y2k_tm);
    
    if (result) {
        printf("Y2K: %d-%02d-%02d %02d:%02d:%02d\n",
               y2k_tm.tm_year + 1900, y2k_tm.tm_mon + 1, y2k_tm.tm_mday,
               y2k_tm.tm_hour, y2k_tm.tm_min, y2k_tm.tm_sec);
    } else {
        printf("Y2K: gmtime_r returned NULL\n");
    }
    
    /* Test Unix epoch */
    printf("\n=== Testing Unix Epoch (0) ===\n");
    time_t epoch = 0;
    struct tm epoch_tm;
    result = gmtime_r(&epoch, &epoch_tm);
    
    if (result) {
        printf("Epoch: %d-%02d-%02d %02d:%02d:%02d\n",
               epoch_tm.tm_year + 1900, epoch_tm.tm_mon + 1, epoch_tm.tm_mday,
               epoch_tm.tm_hour, epoch_tm.tm_min, epoch_tm.tm_sec);
    } else {
        printf("Epoch: gmtime_r returned NULL\n");
    }
    
    printf("\n=== Test Complete ===\n");
    return 0;
}