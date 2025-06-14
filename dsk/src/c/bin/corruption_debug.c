#include <stdio.h>
#include <time.h>
#include <string.h>

/* Our own buffer to test with */
static char our_buffer[sizeof(struct tm)];

/* Test function that mimics gmtime step by step */
struct tm* debug_gmtime(const time_t* timer) {
    struct tm* result = (struct tm*)our_buffer;
    
    printf("DEBUG: Starting debug_gmtime\n");
    printf("DEBUG: Buffer address: %p\n", (void*)our_buffer);
    printf("DEBUG: Result address: %p\n", (void*)result);
    
    if (!timer) {
        printf("DEBUG: timer is NULL\n");
        return NULL;
    }
    
    time_t timestamp = *timer;
    printf("DEBUG: timestamp = %ld\n", (long)timestamp);
    
    /* Clear the buffer */
    printf("DEBUG: Clearing buffer with memset\n");
    memset(our_buffer, 0, sizeof(struct tm));
    printf("DEBUG: Buffer cleared\n");
    
    /* Test: Can we write and read back immediately? */
    printf("DEBUG: Testing immediate write/read\n");
    result->tm_year = 999;
    printf("DEBUG: Set tm_year to 999, reading back: %d\n", result->tm_year);
    
    if (result->tm_year != 999) {
        printf("ERROR: Immediate write/read failed!\n");
        return NULL;
    }
    
    /* For timestamp 946684801: Jan 1, 2000 00:00:01 UTC */
    if (timestamp == 946684801) {
        printf("DEBUG: Hardcoded case for 946684801\n");
        
        printf("DEBUG: Setting tm_sec = 1\n");
        result->tm_sec = 1;
        printf("DEBUG: tm_sec is now: %d\n", result->tm_sec);
        
        printf("DEBUG: Setting tm_min = 0\n");
        result->tm_min = 0;
        printf("DEBUG: tm_min is now: %d\n", result->tm_min);
        
        printf("DEBUG: Setting tm_hour = 0\n");
        result->tm_hour = 0;
        printf("DEBUG: tm_hour is now: %d\n", result->tm_hour);
        
        printf("DEBUG: Setting tm_mday = 1\n");
        result->tm_mday = 1;
        printf("DEBUG: tm_mday is now: %d\n", result->tm_mday);
        
        printf("DEBUG: Setting tm_mon = 0\n");
        result->tm_mon = 0;
        printf("DEBUG: tm_mon is now: %d\n", result->tm_mon);
        
        printf("DEBUG: Setting tm_year = 100\n");
        result->tm_year = 100;
        printf("DEBUG: tm_year is now: %d\n", result->tm_year);
        
        printf("DEBUG: Setting tm_wday = 6\n");
        result->tm_wday = 6;
        printf("DEBUG: tm_wday is now: %d\n", result->tm_wday);
        
        printf("DEBUG: Setting tm_yday = 0\n");
        result->tm_yday = 0;
        printf("DEBUG: tm_yday is now: %d\n", result->tm_yday);
        
        printf("DEBUG: Setting tm_isdst = 0\n");
        result->tm_isdst = 0;
        printf("DEBUG: tm_isdst is now: %d\n", result->tm_isdst);
        
        printf("DEBUG: Final check before return:\n");
        printf("  tm_sec = %d (should be 1)\n", result->tm_sec);
        printf("  tm_min = %d (should be 0)\n", result->tm_min);
        printf("  tm_hour = %d (should be 0)\n", result->tm_hour);
        printf("  tm_mday = %d (should be 1)\n", result->tm_mday);
        printf("  tm_mon = %d (should be 0)\n", result->tm_mon);
        printf("  tm_year = %d (should be 100)\n", result->tm_year);
        printf("  tm_wday = %d (should be 6)\n", result->tm_wday);
        
        printf("DEBUG: About to return %p\n", (void*)result);
        return result;
    }
    
    printf("DEBUG: Not a hardcoded timestamp\n");
    return NULL;
}

int main(void) {
    printf("=== Corruption Debug Test ===\n");
    
    /* Test our debug function */
    time_t test_time = 946684801;
    printf("Calling debug_gmtime(%ld)...\n", (long)test_time);
    
    struct tm* result = debug_gmtime(&test_time);
    
    if (result == NULL) {
        printf("debug_gmtime returned NULL!\n");
        return 1;
    }
    
    printf("\nAfter return from debug_gmtime:\n");
    printf("Returned pointer: %p\n", (void*)result);
    printf("Reading values after return:\n");
    printf("  tm_sec = %d (should be 1)\n", result->tm_sec);
    printf("  tm_min = %d (should be 0)\n", result->tm_min);
    printf("  tm_hour = %d (should be 0)\n", result->tm_hour);
    printf("  tm_mday = %d (should be 1)\n", result->tm_mday);
    printf("  tm_mon = %d (should be 0)\n", result->tm_mon);
    printf("  tm_year = %d (should be 100)\n", result->tm_year);
    printf("  tm_wday = %d (should be 6)\n", result->tm_wday);
    
    /* Test if calling the real gmtime affects our buffer */
    printf("\nNow testing real gmtime...\n");
    printf("Our buffer before calling real gmtime:\n");
    printf("  our tm_year = %d\n", result->tm_year);
    
    struct tm* real_result = gmtime(&test_time);
    printf("Real gmtime returned: %p\n", (void*)real_result);
    
    printf("Our buffer after calling real gmtime:\n");
    printf("  our tm_year = %d\n", result->tm_year);
    
    if (real_result) {
        printf("Real gmtime result:\n");
        printf("  tm_year = %d\n", real_result->tm_year);
        printf("  tm_sec = %d\n", real_result->tm_sec);
    }
    
    printf("\n=== Test Complete ===\n");
    return 0;
}