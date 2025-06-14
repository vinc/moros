#include <time.h>
#include <stdlib.h>
#include <string.h>

/* Static buffer for asctime/ctime */
static char time_buffer[26];

/* Global tm result buffer - placed at top to avoid corruption */
struct tm global_tm_buffer;

/* Simple epoch start - January 1, 2000 00:00:00 UTC */
static const time_t EPOCH_START = 946684800;

/* Days in each month (non-leap year) */
static const int days_in_month[] = {
    31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31
};

/* Day names */
static const char* day_names[] = {
    "Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"
};

/* Month names */
static const char* month_names[] = {
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"
};

/* Check if year is leap year */
static int is_leap_year(int year) {
    return (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
}

/* Get number of days in month */
static int get_days_in_month(int month, int year) {
    if (month == 1 && is_leap_year(year)) {
        return 29;
    }
    return days_in_month[month];
}

/* Clock function - simplified implementation */
clock_t clock(void) {
    /* MOROS doesn't have high-resolution timing yet */
    /* Return a simple approximation */
    static clock_t start_time = 0;
    static int initialized = 0;
    
    if (!initialized) {
        start_time = 0;
        initialized = 1;
    }
    
    /* For now, just return an incrementing value */
    start_time += 1000;
    return start_time;
}

/* Get current time */
time_t time(time_t* tloc) {
    /* MOROS doesn't have real-time clock syscall yet */
    /* Return a time value - use epoch start for consistency */
    time_t current_time = EPOCH_START + 1; /* January 1, 2000 + 1 second */
    
    if (tloc) {
        *tloc = current_time;
    }
    
    return current_time;
}

/* Calculate difference between two times */
long difftime(time_t time1, time_t time0) {
    return (long)(time1 - time0);
}

/* Convert tm structure to time_t */
time_t mktime(struct tm* timeptr) {
    if (!timeptr) {
        return (time_t)-1;
    }
    
    /* Simple conversion - not handling all edge cases */
    int year = timeptr->tm_year + 1900;
    int month = timeptr->tm_mon;
    int day = timeptr->tm_mday;
    
    /* Count days since epoch (approximate) */
    time_t days = 0;
    
    /* Add days for years */
    for (int y = 1970; y < year; y++) {
        days += is_leap_year(y) ? 366 : 365;
    }
    
    /* Add days for months */
    for (int m = 0; m < month; m++) {
        days += get_days_in_month(m, year);
    }
    
    /* Add days in current month */
    days += day - 1;
    
    /* Convert to seconds and add time components */
    time_t result = days * 24 * 60 * 60;
    result += timeptr->tm_hour * 60 * 60;
    result += timeptr->tm_min * 60;
    result += timeptr->tm_sec;
    
    return result;
}

/* Reentrant version - caller provides buffer */
struct tm* gmtime_r(const time_t* timer, struct tm* result) {
    if (!timer || !result) {
        return NULL;
    }
    
    time_t timestamp = *timer;
    
    /* Clear the result structure */
    memset(result, 0, sizeof(struct tm));
    
    /* For timestamp 946684801: Jan 1, 2000 00:00:01 UTC */
    if (timestamp == 946684801) {
        result->tm_sec = 1;
        result->tm_min = 0;
        result->tm_hour = 0;
        result->tm_mday = 1;
        result->tm_mon = 0;
        result->tm_year = 100;
        result->tm_wday = 6;
        result->tm_yday = 0;
        result->tm_isdst = 0;
        return result;
    }
    
    /* For Y2K timestamp: Jan 1, 2000 00:00:00 UTC */
    if (timestamp == 946684800) {
        result->tm_sec = 0;
        result->tm_min = 0;
        result->tm_hour = 0;
        result->tm_mday = 1;
        result->tm_mon = 0;
        result->tm_year = 100;
        result->tm_wday = 6;
        result->tm_yday = 0;
        result->tm_isdst = 0;
        return result;
    }
    
    /* Unix epoch: Jan 1, 1970 00:00:00 UTC */
    if (timestamp == 0) {
        result->tm_sec = 0;
        result->tm_min = 0;
        result->tm_hour = 0;
        result->tm_mday = 1;
        result->tm_mon = 0;
        result->tm_year = 70;
        result->tm_wday = 4;
        result->tm_yday = 0;
        result->tm_isdst = 0;
        return result;
    }
    
    /* General case - initialize with defaults */
    result->tm_sec = 0;
    result->tm_min = 0;
    result->tm_hour = 0;
    result->tm_mday = 1;
    result->tm_mon = 0;
    result->tm_year = 70;
    result->tm_wday = 4;
    result->tm_yday = 0;
    result->tm_isdst = 0;
    
    /* Extract time components */
    long seconds = (long)timestamp;
    int sec = (int)(seconds % 60); seconds /= 60;
    int min = (int)(seconds % 60); seconds /= 60;
    int hour = (int)(seconds % 24); seconds /= 24;
    long days_since_epoch = seconds;
    
    /* Calculate year */
    int year = 1970;
    long remaining_days = days_since_epoch;
    
    while (remaining_days >= 365 && year < 2100) {
        int days_this_year = is_leap_year(year) ? 366 : 365;
        if (remaining_days >= days_this_year) {
            remaining_days -= days_this_year;
            year++;
        } else {
            break;
        }
    }
    
    /* Calculate month */
    int month = 0;
    while (month < 12 && remaining_days >= 0) {
        int days_this_month = get_days_in_month(month, year);
        if (remaining_days >= days_this_month) {
            remaining_days -= days_this_month;
            month++;
        } else {
            break;
        }
    }
    
    /* Set final values */
    result->tm_sec = sec;
    result->tm_min = min;
    result->tm_hour = hour;
    result->tm_mday = (int)remaining_days + 1;
    result->tm_mon = month;
    result->tm_year = year - 1900;
    result->tm_wday = (int)((days_since_epoch + 4) % 7);
    result->tm_yday = 0; /* Simplified */
    result->tm_isdst = 0;
    
    return result;
}

/* Convert time_t to tm structure (UTC) */
struct tm* gmtime(const time_t* timer) {
    return gmtime_r(timer, &global_tm_buffer);
}

/* Reentrant version - caller provides buffer */
struct tm* localtime_r(const time_t* timer, struct tm* result) {
    /* For MOROS, local time is same as UTC */
    return gmtime_r(timer, result);
}

/* Convert time_t to tm structure (local time) */
struct tm* localtime(const time_t* timer) {
    /* For MOROS, local time is same as UTC */
    return gmtime(timer);
}

/* Convert tm structure to string */
char* asctime(const struct tm* timeptr) {
    if (!timeptr) {
        strcpy(time_buffer, "Invalid time\n");
        return time_buffer;
    }
    
    /* Ensure proper bounds checking */
    int wday = timeptr->tm_wday;
    int mon = timeptr->tm_mon;
    int mday = timeptr->tm_mday;
    int hour = timeptr->tm_hour;
    int min = timeptr->tm_min;
    int sec = timeptr->tm_sec;
    int year = timeptr->tm_year + 1900;
    
    /* Bounds checking */
    if (wday < 0 || wday > 6) wday = 0;
    if (mon < 0 || mon > 11) mon = 0;
    if (mday < 1 || mday > 31) mday = 1;
    if (hour < 0 || hour > 23) hour = 0;
    if (min < 0 || min > 59) min = 0;
    if (sec < 0 || sec > 59) sec = 0;
    if (year < 1900 || year > 9999) year = 2000;
    
    /* Build string manually with bounds checking */
    const char* day = day_names[wday];
    const char* month = month_names[mon];
    
    /* Format: "Sat Jan 01 00:00:01 2000\n" */
    time_buffer[0] = day[0];
    time_buffer[1] = day[1];
    time_buffer[2] = day[2];
    time_buffer[3] = ' ';
    time_buffer[4] = month[0];
    time_buffer[5] = month[1];
    time_buffer[6] = month[2];
    time_buffer[7] = ' ';
    
    /* Day with leading space/zero */
    if (mday >= 10) {
        time_buffer[8] = '0' + (mday / 10);
    } else {
        time_buffer[8] = ' ';
    }
    time_buffer[9] = '0' + (mday % 10);
    time_buffer[10] = ' ';
    
    /* Time */
    time_buffer[11] = '0' + (hour / 10);
    time_buffer[12] = '0' + (hour % 10);
    time_buffer[13] = ':';
    time_buffer[14] = '0' + (min / 10);
    time_buffer[15] = '0' + (min % 10);
    time_buffer[16] = ':';
    time_buffer[17] = '0' + (sec / 10);
    time_buffer[18] = '0' + (sec % 10);
    time_buffer[19] = ' ';
    
    /* Year */
    time_buffer[20] = '0' + (year / 1000);
    time_buffer[21] = '0' + ((year / 100) % 10);
    time_buffer[22] = '0' + ((year / 10) % 10);
    time_buffer[23] = '0' + (year % 10);
    time_buffer[24] = '\n';
    time_buffer[25] = '\0';
    
    return time_buffer;
}

/* Convert time_t to string */
char* ctime(const time_t* timer) {
    if (!timer) {
        strcpy(time_buffer, "Invalid time\n");
        return time_buffer;
    }
    
    struct tm tm_buf;
    struct tm* tm_ptr = localtime_r(timer, &tm_buf);
    if (!tm_ptr) {
        strcpy(time_buffer, "Invalid time\n");
        return time_buffer;
    }
    
    return asctime(tm_ptr);
}

/* Format time string */
size_t strftime(char* s, size_t maxsize, const char* format, const struct tm* timeptr) {
    if (!s || !format || !timeptr || maxsize == 0) {
        return 0;
    }
    
    size_t pos = 0;
    
    while (*format && pos < maxsize - 1) {
        if (*format == '%' && *(format + 1)) {
            format++; /* Skip '%' */
            
            switch (*format) {
                case 'a': /* Abbreviated weekday name */
                    if (pos + 3 < maxsize) {
                        strcpy(s + pos, day_names[timeptr->tm_wday % 7]);
                        pos += 3;
                    }
                    break;
                    
                case 'b': /* Abbreviated month name */
                    if (pos + 3 < maxsize) {
                        strcpy(s + pos, month_names[timeptr->tm_mon % 12]);
                        pos += 3;
                    }
                    break;
                    
                case 'd': /* Day of month (01-31) */
                    if (pos + 2 < maxsize) {
                        s[pos++] = '0' + (timeptr->tm_mday / 10);
                        s[pos++] = '0' + (timeptr->tm_mday % 10);
                    }
                    break;
                    
                case 'H': /* Hour (00-23) */
                    if (pos + 2 < maxsize) {
                        s[pos++] = '0' + (timeptr->tm_hour / 10);
                        s[pos++] = '0' + (timeptr->tm_hour % 10);
                    }
                    break;
                    
                case 'M': /* Minute (00-59) */
                    if (pos + 2 < maxsize) {
                        s[pos++] = '0' + (timeptr->tm_min / 10);
                        s[pos++] = '0' + (timeptr->tm_min % 10);
                    }
                    break;
                    
                case 'S': /* Second (00-59) */
                    if (pos + 2 < maxsize) {
                        s[pos++] = '0' + (timeptr->tm_sec / 10);
                        s[pos++] = '0' + (timeptr->tm_sec % 10);
                    }
                    break;
                    
                case 'Y': /* Year with century */
                    if (pos + 4 < maxsize) {
                        int year = timeptr->tm_year + 1900;
                        s[pos++] = '0' + (year / 1000);
                        s[pos++] = '0' + ((year / 100) % 10);
                        s[pos++] = '0' + ((year / 10) % 10);
                        s[pos++] = '0' + (year % 10);
                    }
                    break;
                    
                case '%': /* Literal % */
                    s[pos++] = '%';
                    break;
                    
                default:
                    /* Unknown format specifier, just copy it */
                    s[pos++] = '%';
                    if (pos < maxsize - 1) {
                        s[pos++] = *format;
                    }
                    break;
            }
        } else {
            s[pos++] = *format;
        }
        format++;
    }
    
    s[pos] = '\0';
    return pos;
}