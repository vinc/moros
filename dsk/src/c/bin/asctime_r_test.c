#include <stdio.h>
#include <time.h>
#include <string.h>

int main(void) {
    printf("=== asctime_r Test (Caller Provides Buffer) ===\n");
    
    /* Test timestamp 946684801: Jan 1, 2000 00:00:01 UTC */
    time_t test_time = 946684801;
    printf("Testing timestamp: %ld\n", (long)test_time);
    
    /* Get the tm structure using localtime_r */
    struct tm my_tm;
    struct tm* tm_result = localtime_r(&test_time, &my_tm);
    
    if (!tm_result) {
        printf("ERROR: localtime_r failed!\n");
        return 1;
    }
    
    printf("localtime_r success: Year %d, Month %d, Day %d, %02d:%02d:%02d, wday=%d\n",
           my_tm.tm_year + 1900, my_tm.tm_mon + 1, my_tm.tm_mday,
           my_tm.tm_hour, my_tm.tm_min, my_tm.tm_sec, my_tm.tm_wday);
    
    /* Test asctime_r with our own buffer */
    printf("\nTesting asctime_r with caller-provided buffer:\n");
    char my_buffer[26];
    printf("Our buffer address: %p\n", (void*)my_buffer);
    
    /* Initialize buffer with test pattern */
    memset(my_buffer, 'X', 25);
    my_buffer[25] = '\0';
    printf("Buffer initialized with X's: '%.10s...'\n", my_buffer);
    
    /* Call asctime_r */
    char* result = asctime_r(&my_tm, my_buffer);
    
    if (result == NULL) {
        printf("ERROR: asctime_r returned NULL!\n");
        return 1;
    }
    
    printf("asctime_r returned: %p\n", (void*)result);
    printf("Our buffer address: %p\n", (void*)my_buffer);
    
    if (result != my_buffer) {
        printf("WARNING: Returned pointer doesn't match our buffer!\n");
    }
    
    printf("\nResult from asctime_r:\n");
    printf("String: '%s'", result); /* asctime includes newline */
    printf("Length: %d\n", (int)strlen(result));
    
    /* Check if it matches expected format */
    if (strlen(result) > 0) {
        printf("✓ asctime_r SUCCESS! Got non-empty string\n");
        
        /* Check for expected content */
        if (result[0] == 'S' && result[1] == 'a' && result[2] == 't') {
            printf("✓ Starts with 'Sat' - correct day!\n");
        } else {
            printf("✗ Does not start with 'Sat'\n");
        }
        
        if (strstr(result, "2000")) {
            printf("✓ Contains '2000' - correct year!\n");
        } else {
            printf("✗ Does not contain '2000'\n");
        }
        
        if (strstr(result, "Jan")) {
            printf("✓ Contains 'Jan' - correct month!\n");
        } else {
            printf("✗ Does not contain 'Jan'\n");
        }
    } else {
        printf("✗ asctime_r FAILED! Got empty string\n");
    }
    
    /* Test multiple calls with different buffers */
    printf("\n=== Testing Multiple asctime_r Calls ===\n");
    
    /* Test Y2K */
    time_t y2k = 946684800;
    struct tm y2k_tm;
    char y2k_buffer[26];
    
    if (localtime_r(&y2k, &y2k_tm)) {
        char* y2k_result = asctime_r(&y2k_tm, y2k_buffer);
        printf("Y2K asctime_r: '%s'", y2k_result ? y2k_result : "NULL");
    }
    
    /* Test Unix epoch */
    time_t epoch = 0;
    struct tm epoch_tm;
    char epoch_buffer[26];
    
    if (localtime_r(&epoch, &epoch_tm)) {
        char* epoch_result = asctime_r(&epoch_tm, epoch_buffer);
        printf("Epoch asctime_r: '%s'", epoch_result ? epoch_result : "NULL");
    }
    
    /* Test if original buffer is still intact */
    printf("\nChecking if original buffer is still valid:\n");
    printf("Original result: '%s'", my_buffer);
    
    /* Test regular asctime for comparison */
    printf("\n=== Comparing with regular asctime ===\n");
    char* asctime_result = asctime(&my_tm);
    printf("Regular asctime: '%s'", asctime_result ? asctime_result : "NULL");
    printf("asctime length: %d\n", asctime_result ? (int)strlen(asctime_result) : 0);
    
    if (asctime_result && strlen(asctime_result) > 0) {
        printf("✓ Regular asctime also works!\n");
    } else {
        printf("✗ Regular asctime still fails\n");
    }
    
    printf("\n=== Test Complete ===\n");
    return 0;
}