#ifndef _TIME_H
#define _TIME_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Time types */
#ifndef _TIME_T_DEFINED
#define _TIME_T_DEFINED
typedef long time_t;
#endif

#ifndef _CLOCK_T_DEFINED
#define _CLOCK_T_DEFINED
typedef long clock_t;
#endif

/* Clock ticks per second */
#define CLOCKS_PER_SEC 1000000L

/* Time structure */
struct tm {
    int tm_sec;    /* Seconds (0-60) */
    int tm_min;    /* Minutes (0-59) */
    int tm_hour;   /* Hours (0-23) */
    int tm_mday;   /* Day of the month (1-31) */
    int tm_mon;    /* Month (0-11) */
    int tm_year;   /* Year - 1900 */
    int tm_wday;   /* Day of the week (0-6, Sunday = 0) */
    int tm_yday;   /* Day in the year (0-365, 1 Jan = 0) */
    int tm_isdst;  /* Daylight saving time */
};

/* Time manipulation functions */
clock_t clock(void);
time_t time(time_t* tloc);
long difftime(time_t time1, time_t time0);
time_t mktime(struct tm* timeptr);

/* Time conversion functions */
char* asctime(const struct tm* timeptr);
char* ctime(const time_t* timer);
struct tm* gmtime(const time_t* timer);
struct tm* localtime(const time_t* timer);

/* Formatted time functions */
size_t strftime(char* s, size_t maxsize, const char* format, const struct tm* timeptr);

#ifdef __cplusplus
}
#endif

#endif /* _TIME_H */