#include <stdio.h>
#include <time.h>

int main(void) {
    printf("=== Direct gmtime Test ===\n");
    
    /* Test the specific timestamp */
    time_t test_time = 946684801;
    printf("Input timestamp: %ld\n", (long)test_time);
    
    /* Call gmtime directly */
    struct tm* result = gmtime(&test_time);
    
    if (result == NULL) {
        printf("ERROR: gmtime returned NULL!\n");
        return 1;
    }
    
    printf("gmtime returned a valid pointer\n");
    printf("Raw values from gmtime:\n");
    printf("  tm_sec = %d\n", result->tm_sec);
    printf("  tm_min = %d\n", result->tm_min);
    printf("  tm_hour = %d\n", result->tm_hour);
    printf("  tm_mday = %d\n", result->tm_mday);
    printf("  tm_mon = %d\n", result->tm_mon);
    printf("  tm_year = %d\n", result->tm_year);
    printf("  tm_wday = %d\n", result->tm_wday);
    printf("  tm_yday = %d\n", result->tm_yday);
    printf("  tm_isdst = %d\n", result->tm_isdst);
    
    /* Expected values */
    printf("\nExpected values:\n");
    printf("  tm_sec = 1\n");
    printf("  tm_min = 0\n");
    printf("  tm_hour = 0\n");
    printf("  tm_mday = 1\n");
    printf("  tm_mon = 0\n");
    printf("  tm_year = 100\n");
    printf("  tm_wday = 6\n");
    
    /* Manual calculation check */
    printf("\nManual calculation:\n");
    time_t t = test_time;
    int sec = t % 60; t /= 60;
    int min = t % 60; t /= 60;
    int hour = t % 24; t /= 24;
    long days = t;
    
    printf("  Seconds: %d\n", sec);
    printf("  Minutes: %d\n", min);
    printf("  Hours: %d\n", hour);
    printf("  Days since epoch: %ld\n", days);
    
    /* Test another timestamp */
    printf("\n=== Testing Y2K (946684800) ===\n");
    time_t y2k = 946684800;
    result = gmtime(&y2k);
    if (result) {
        printf("Y2K: Year %d, Month %d, Day %d, %02d:%02d:%02d\n",
               result->tm_year + 1900, result->tm_mon + 1, result->tm_mday,
               result->tm_hour, result->tm_min, result->tm_sec);
    } else {
        printf("Y2K: gmtime returned NULL\n");
    }
    
    /* Test Unix epoch */
    printf("\n=== Testing Unix Epoch (0) ===\n");
    time_t epoch = 0;
    result = gmtime(&epoch);
    if (result) {
        printf("Epoch: Year %d, Month %d, Day %d, %02d:%02d:%02d\n",
               result->tm_year + 1900, result->tm_mon + 1, result->tm_mday,
               result->tm_hour, result->tm_min, result->tm_sec);
    } else {
        printf("Epoch: gmtime returned NULL\n");
    }
    
    printf("\n=== Test Complete ===\n");
    return 0;
}