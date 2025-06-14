#ifndef TEST_FRAMEWORK_H
#define TEST_FRAMEWORK_H

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* ANSI color codes for test output */
#define TEST_COLOR_GREEN "\033[32m"
#define TEST_COLOR_RED   "\033[31m"
#define TEST_COLOR_RESET "\033[0m"

/* Test result tracking */
extern int test_count;
extern int test_passed;
extern int test_failed;

/* Test function pointer type */
typedef void (*test_func_t)(void);

/* Test case structure */
typedef struct {
    const char* name;
    test_func_t func;
} test_case_t;

/* Assertion macros */
#define ASSERT_EQ(expected, actual) \
    do { \
        if ((expected) != (actual)) { \
            printf("\n%spanicked at %s:%d:%d:\n", TEST_COLOR_RED, __FILE__, __LINE__, 5); \
            printf("assertion `left == right` failed\n"); \
            printf("  left: \"%ld\"\n", (long)(expected)); \
            printf(" right: \"%ld\"%s\n", (long)(actual), TEST_COLOR_RESET); \
            fflush(stdout); \
            test_fail(); \
            return; \
        } \
    } while(0)

#define ASSERT_STR_EQ(expected, actual) \
    do { \
        if (strcmp((expected), (actual)) != 0) { \
            printf("\n%spanicked at %s:%d:%d:\n", TEST_COLOR_RED, __FILE__, __LINE__, 5); \
            printf("assertion `left == right` failed\n"); \
            printf("  left: \"%s\"\n", (expected)); \
            printf(" right: \"%s\"%s\n", (actual), TEST_COLOR_RESET); \
            fflush(stdout); \
            test_fail(); \
            return; \
        } \
    } while(0)

#define ASSERT_NOT_NULL(ptr) \
    do { \
        if ((ptr) == NULL) { \
            printf("\n%spanicked at %s:%d:%d:\n", TEST_COLOR_RED, __FILE__, __LINE__, 5); \
            printf("assertion failed: pointer is NULL%s\n", TEST_COLOR_RESET); \
            fflush(stdout); \
            test_fail(); \
            return; \
        } \
    } while(0)

#define ASSERT_NULL(ptr) \
    do { \
        if ((ptr) != NULL) { \
            printf("\n%spanicked at %s:%d:%d:\n", TEST_COLOR_RED, __FILE__, __LINE__, 5); \
            printf("assertion failed: pointer is not NULL%s\n", TEST_COLOR_RESET); \
            fflush(stdout); \
            test_fail(); \
            return; \
        } \
    } while(0)

#define ASSERT_TRUE(condition) \
    do { \
        if (!(condition)) { \
            printf("\n%spanicked at %s:%d:%d:\n", TEST_COLOR_RED, __FILE__, __LINE__, 5); \
            printf("assertion failed: condition is false%s\n", TEST_COLOR_RESET); \
            fflush(stdout); \
            test_fail(); \
            return; \
        } \
    } while(0)

#define ASSERT_FALSE(condition) \
    do { \
        if ((condition)) { \
            printf("\n%spanicked at %s:%d:%d:\n", TEST_COLOR_RED, __FILE__, __LINE__, 5); \
            printf("assertion failed: condition is true%s\n", TEST_COLOR_RESET); \
            fflush(stdout); \
            test_fail(); \
            return; \
        } \
    } while(0)

/* Test registration and execution */
void run_test(const char* test_name, test_func_t test_func);
void run_all_tests(test_case_t* tests, int num_tests);
void test_summary(void);

/* Test result functions */
void test_init(void);
void test_pass(void);
void test_fail(void);

#endif /* TEST_FRAMEWORK_H */