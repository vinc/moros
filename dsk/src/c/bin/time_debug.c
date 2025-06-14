#include <stdio.h>
#include <time.h>

int main(void) {
    printf("=== Time Debug Test ===\n");
    
    /* Test the specific timestamp we're seeing */
    time_t test_time = 946684801;
    printf("Testing timestamp: %ld\n", (long)test_time);
    printf("This should be: Jan 1, 2000 00:00:01 UTC\n\n");
    
    /* Manual calculation step by step */
    printf("Manual calculation:\n");
    time_t t = test_time;
    
    int sec = (int)(t % 60);
    t /= 60;
    printf("  Seconds: %d\n", sec);
    
    int min = (int)(t % 60);
    t /= 60;
    printf("  Minutes: %d\n", min);
    
    int hour = (int)(t % 24);
    t /= 24;
    printf("  Hours: %d\n", hour);
    
    long total_days = t;
    printf("  Total days since epoch: %ld\n", total_days);
    
    /* Calculate years manually */
    int year = 1970;
    long days_remaining = total_days;
    int days_in_year;
    
    printf("\nYear calculation:\n");
    while (days_remaining > 0) {
        /* Simple leap year check */
        int is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        days_in_year = is_leap ? 366 : 365;
        
        printf("  Year %d: %d days, remaining: %ld\n", year, days_in_year, days_remaining);
        
        if (days_remaining >= days_in_year) {
            days_remaining -= days_in_year;
            year++;
        } else {
            break;
        }
        
        /* Safety check */
        if (year > 2010) {
            printf("  Safety break at year %d\n", year);
            break;
        }
    }
    
    printf("  Final year: %d (tm_year = %d)\n", year, year - 1900);
    printf("  Days into year: %ld\n", days_remaining);
    
    /* Calculate month */
    int days_in_month[] = {31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31};
    int month = 0;
    
    /* Adjust February for leap year */
    int is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    if (is_leap) days_in_month[1] = 29;
    
    printf("\nMonth calculation (leap year: %s):\n", is_leap ? "yes" : "no");
    while (days_remaining > 0 && month < 12) {
        printf("  Month %d: %d days, remaining: %ld\n", month + 1, days_in_month[month], days_remaining);
        
        if (days_remaining >= days_in_month[month]) {
            days_remaining -= days_in_month[month];
            month++;
        } else {
            break;
        }
    }
    
    int day = (int)days_remaining + 1;
    printf("  Final month: %d (tm_mon = %d)\n", month + 1, month);
    printf("  Day of month: %d\n", day);
    
    /* Test with actual localtime function */
    printf("\nUsing actual localtime function:\n");
    struct tm* tm_result = localtime(&test_time);
    if (tm_result) {
        printf("  tm_year: %d (year: %d)\n", tm_result->tm_year, tm_result->tm_year + 1900);
        printf("  tm_mon: %d (month: %d)\n", tm_result->tm_mon, tm_result->tm_mon + 1);
        printf("  tm_mday: %d\n", tm_result->tm_mday);
        printf("  tm_hour: %d\n", tm_result->tm_hour);
        printf("  tm_min: %d\n", tm_result->tm_min);
        printf("  tm_sec: %d\n", tm_result->tm_sec);
        printf("  tm_wday: %d\n", tm_result->tm_wday);
    } else {
        printf("  localtime returned NULL!\n");
    }
    
    /* Test ctime */
    printf("\nUsing ctime:\n");
    char* time_str = ctime(&test_time);
    if (time_str) {
        printf("  ctime result: '%s'\n", time_str);
    } else {
        printf("  ctime returned NULL!\n");
    }
    
    printf("\n=== Expected Result ===\n");
    printf("Should be: Sat Jan 01 00:00:01 2000\n");
    printf("=== Time Debug Complete ===\n");
    
    return 0;
}