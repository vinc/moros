#include <stdio.h>
#include <time.h>
#include <string.h>

int main(void) {
    printf("=== ctime Debug Test ===\n");
    
    time_t test_time = 946684801; /* Jan 1, 2000 00:00:01 UTC */
    printf("Testing timestamp: %ld\n", (long)test_time);
    
    /* Test 1: Check if localtime_r works */
    printf("\nTesting localtime_r first:\n");
    struct tm my_tm;
    struct tm* tm_result = localtime_r(&test_time, &my_tm);
    
    if (tm_result) {
        printf("localtime_r: Year %d, Month %d, Day %d, %02d:%02d:%02d, wday=%d\n",
               my_tm.tm_year + 1900, my_tm.tm_mon + 1, my_tm.tm_mday,
               my_tm.tm_hour, my_tm.tm_min, my_tm.tm_sec, my_tm.tm_wday);
    } else {
        printf("localtime_r failed!\n");
        return 1;
    }
    
    /* Test 2: Test asctime directly */
    printf("\nTesting asctime directly:\n");
    char* asctime_result = asctime(&my_tm);
    if (asctime_result) {
        printf("asctime result: '%s'\n", asctime_result);
        printf("asctime length: %d\n", (int)strlen(asctime_result));
        
        /* Check each character */
        printf("First 10 chars: ");
        for (int i = 0; i < 10 && asctime_result[i]; i++) {
            printf("[%d='%c'] ", asctime_result[i], asctime_result[i]);
        }
        printf("\n");
    } else {
        printf("asctime returned NULL!\n");
    }
    
    /* Test 3: Test ctime */
    printf("\nTesting ctime:\n");
    char* ctime_result = ctime(&test_time);
    if (ctime_result) {
        printf("ctime result: '%s'\n", ctime_result);
        printf("ctime length: %d\n", (int)strlen(ctime_result));
        
        if (strlen(ctime_result) == 0) {
            printf("ctime returned empty string!\n");
        } else {
            printf("First 10 chars: ");
            for (int i = 0; i < 10 && ctime_result[i]; i++) {
                printf("[%d='%c'] ", ctime_result[i], ctime_result[i]);
            }
            printf("\n");
        }
    } else {
        printf("ctime returned NULL!\n");
    }
    
    /* Test 4: Manual asctime test */
    printf("\nManual asctime test with known values:\n");
    struct tm manual_tm;
    manual_tm.tm_sec = 1;
    manual_tm.tm_min = 0;
    manual_tm.tm_hour = 0;
    manual_tm.tm_mday = 1;
    manual_tm.tm_mon = 0;
    manual_tm.tm_year = 100;
    manual_tm.tm_wday = 6;
    manual_tm.tm_yday = 0;
    manual_tm.tm_isdst = 0;
    
    char* manual_asctime = asctime(&manual_tm);
    if (manual_asctime) {
        printf("Manual asctime: '%s'\n", manual_asctime);
        printf("Should be: 'Sat Jan 01 00:00:01 2000\\n'\n");
    } else {
        printf("Manual asctime returned NULL!\n");
    }
    
    /* Test 5: Check if the issue is with the static buffer */
    printf("\nTesting multiple ctime calls:\n");
    time_t times[] = {0, 946684800, 946684801};
    char* names[] = {"Epoch", "Y2K", "Y2K+1"};
    
    for (int i = 0; i < 3; i++) {
        char* result = ctime(&times[i]);
        printf("%s ctime: '%s' (len=%d)\n", names[i], 
               result ? result : "NULL", 
               result ? (int)strlen(result) : 0);
    }
    
    printf("\n=== Test Complete ===\n");
    return 0;
}