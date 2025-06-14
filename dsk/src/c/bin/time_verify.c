#include <stdio.h>
#include <time.h>

int main(void) {
    printf("=== Time Verification Test ===\n");
    
    /* Test the specific timestamp that was failing */
    time_t test_time = 946684801; /* Jan 1, 2000 00:00:01 UTC */
    
    printf("Testing timestamp: %ld\n", (long)test_time);
    printf("Expected: Sat Jan 01 00:00:01 2000\n\n");
    
    /* Test localtime */
    struct tm* tm_result = localtime(&test_time);
    if (tm_result) {
        printf("localtime results:\n");
        printf("  Year: %d (tm_year: %d)\n", tm_result->tm_year + 1900, tm_result->tm_year);
        printf("  Month: %d (tm_mon: %d)\n", tm_result->tm_mon + 1, tm_result->tm_mon);
        printf("  Day: %d\n", tm_result->tm_mday);
        printf("  Hour: %d\n", tm_result->tm_hour);
        printf("  Minute: %d\n", tm_result->tm_min);
        printf("  Second: %d\n", tm_result->tm_sec);
        printf("  Weekday: %d (0=Sun, 6=Sat)\n", tm_result->tm_wday);
        
        /* Verify expected values */
        int correct = 1;
        if (tm_result->tm_year != 100) { printf("ERROR: Year should be 100, got %d\n", tm_result->tm_year); correct = 0; }
        if (tm_result->tm_mon != 0) { printf("ERROR: Month should be 0, got %d\n", tm_result->tm_mon); correct = 0; }
        if (tm_result->tm_mday != 1) { printf("ERROR: Day should be 1, got %d\n", tm_result->tm_mday); correct = 0; }
        if (tm_result->tm_hour != 0) { printf("ERROR: Hour should be 0, got %d\n", tm_result->tm_hour); correct = 0; }
        if (tm_result->tm_min != 0) { printf("ERROR: Minute should be 0, got %d\n", tm_result->tm_min); correct = 0; }
        if (tm_result->tm_sec != 1) { printf("ERROR: Second should be 1, got %d\n", tm_result->tm_sec); correct = 0; }
        if (tm_result->tm_wday != 6) { printf("ERROR: Weekday should be 6 (Sat), got %d\n", tm_result->tm_wday); correct = 0; }
        
        if (correct) {
            printf("\n✓ All values are CORRECT!\n");
        } else {
            printf("\n✗ Some values are INCORRECT!\n");
        }
    } else {
        printf("ERROR: localtime returned NULL!\n");
    }
    
    /* Test ctime */
    printf("\nTesting ctime:\n");
    char* time_str = ctime(&test_time);
    if (time_str) {
        printf("ctime result: '%s'", time_str); /* ctime includes newline */
        
        /* Simple check if it looks right */
        if (time_str[0] == 'S' && time_str[1] == 'a' && time_str[2] == 't') {
            printf("✓ Starts with 'Sat' - looks good!\n");
        } else {
            printf("✗ Does not start with 'Sat'\n");
        }
    } else {
        printf("ERROR: ctime returned NULL!\n");
    }
    
    /* Test a few more timestamps */
    printf("\n=== Testing Additional Timestamps ===\n");
    
    time_t epoch = 0; /* Jan 1, 1970 00:00:00 UTC */
    printf("Unix epoch (0): ");
    tm_result = localtime(&epoch);
    if (tm_result) {
        printf("Year %d, Month %d, Day %d\n", 
               tm_result->tm_year + 1900, tm_result->tm_mon + 1, tm_result->tm_mday);
    } else {
        printf("NULL\n");
    }
    
    time_t y2k = 946684800; /* Jan 1, 2000 00:00:00 UTC */
    printf("Y2K (946684800): ");
    tm_result = localtime(&y2k);
    if (tm_result) {
        printf("Year %d, Month %d, Day %d, Hour %d, Min %d, Sec %d\n", 
               tm_result->tm_year + 1900, tm_result->tm_mon + 1, tm_result->tm_mday,
               tm_result->tm_hour, tm_result->tm_min, tm_result->tm_sec);
    } else {
        printf("NULL\n");
    }
    
    printf("\n=== Test Complete ===\n");
    return 0;
}