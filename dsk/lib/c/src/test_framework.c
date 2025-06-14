#include "test_framework.h"

/* Global test counters */
int test_count = 0;
int test_passed = 0;
int test_failed = 0;

/* Initialize test counters */
void test_init(void) {
    test_count = 0;
    test_passed = 0;
    test_failed = 0;
}

/* Mark a test as passed */
void test_pass(void) {
    test_passed++;
}

/* Mark a test as failed */
void test_fail(void) {
    test_failed++;
}

/* Run a single test */
void run_test(const char* test_name, test_func_t test_func) {
    test_count++;
    printf("test %s ... ", test_name);
    
    /* Save current test state */
    int initial_failed = test_failed;
    
    /* Run the test */
    test_func();
    
    /* Check if test failed during execution */
    if (test_failed > initial_failed) {
        /* Test already printed failure details, don't print ok */
        return;
    }
    
    /* Test passed */
    test_pass();
    printf("%sok%s\n", TEST_COLOR_GREEN, TEST_COLOR_RESET);
}

/* Run all tests in an array */
void run_all_tests(test_case_t* tests, int num_tests) {
    printf("\nrunning %d test%s\n", num_tests, num_tests == 1 ? "" : "s");
    
    for (int i = 0; i < num_tests; i++) {
        run_test(tests[i].name, tests[i].func);
    }
    
    test_summary();
}

/* Print test summary */
void test_summary(void) {
    printf("\ntest result: ");
    
    if (test_failed > 0) {
        printf("%sFAILED%s. ", TEST_COLOR_RED, TEST_COLOR_RESET);
        printf("%d passed; %d failed\n", test_passed, test_failed);
    } else {
        printf("%sok%s. %d passed; 0 failed\n", 
               TEST_COLOR_GREEN, TEST_COLOR_RESET, test_passed);
    }
}