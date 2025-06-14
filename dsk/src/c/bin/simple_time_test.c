#include <stdio.h>
#include <time.h>

int main(void) {
    printf("=== Simple Time Test ===\n");
    
    /* Test the hardcoded case */
    time_t test_time = 946684801;
    printf("Testing hardcoded case: %ld\n", (long)test_time);
    
    struct tm* result = localtime(&test_time);
    if (result) {
        printf("SUCCESS: localtime returned a result\n");
        printf("  Year: %d (should be 2000)\n", result->tm_year + 1900);
        printf("  Month: %d (should be 1)\n", result->tm_mon + 1);
        printf("  Day: %d (should be 1)\n", result->tm_mday);
        printf("  Hour: %d (should be 0)\n", result->tm_hour);
        printf("  Min: %d (should be 0)\n", result->tm_min);
        printf("  Sec: %d (should be 1)\n", result->tm_sec);
        printf("  Weekday: %d (should be 6 for Saturday)\n", result->tm_wday);
        
        /* Check if values are correct */
        if (result->tm_year == 100 && result->tm_mon == 0 && result->tm_mday == 1 &&
            result->tm_hour == 0 && result->tm_min == 0 && result->tm_sec == 1) {
            printf("✓ HARDCODED CASE WORKS!\n");
        } else {
            printf("✗ Hardcoded case failed\n");
        }
    } else {
        printf("FAILED: localtime returned NULL\n");
    }
    
    /* Test ctime */
    printf("\nTesting ctime:\n");
    char* time_str = ctime(&test_time);
    if (time_str && time_str[0] != '\0') {
        printf("ctime result: %s", time_str);
        printf("✓ ctime works!\n");
    } else {
        printf("✗ ctime failed\n");
    }
    
    /* Test Y2K */
    printf("\nTesting Y2K (946684800):\n");
    time_t y2k = 946684800;
    result = localtime(&y2k);
    if (result) {
        printf("Y2K: %d-%02d-%02d %02d:%02d:%02d\n",
               result->tm_year + 1900, result->tm_mon + 1, result->tm_mday,
               result->tm_hour, result->tm_min, result->tm_sec);
    } else {
        printf("Y2K: NULL\n");
    }
    
    printf("\n=== Test Complete ===\n");
    return 0;
}