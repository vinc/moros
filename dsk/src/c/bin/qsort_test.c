#include <stdio.h>
#include <stdlib.h>

/* Comparison function for qsort */
int compare_ints(const void* a, const void* b) {
    return (*(int*)a - *(int*)b);
}

int main(void) {
    printf("=== QSort Test ===\n");
    
    /* Test 1: Simple array */
    printf("Test 1: Simple array sorting\n");
    int nums1[] = {5, 2, 8, 1, 9, 3};
    int count1 = sizeof(nums1) / sizeof(nums1[0]);
    
    printf("  Before: ");
    for (int i = 0; i < count1; i++) {
        printf("%d ", nums1[i]);
    }
    printf("\n");
    
    qsort(nums1, count1, sizeof(int), compare_ints);
    
    printf("  After:  ");
    for (int i = 0; i < count1; i++) {
        printf("%d ", nums1[i]);
    }
    printf("\n");
    
    /* Test 2: Already sorted array */
    printf("\nTest 2: Already sorted array\n");
    int nums2[] = {1, 2, 3, 4, 5};
    int count2 = sizeof(nums2) / sizeof(nums2[0]);
    
    printf("  Before: ");
    for (int i = 0; i < count2; i++) {
        printf("%d ", nums2[i]);
    }
    printf("\n");
    
    qsort(nums2, count2, sizeof(int), compare_ints);
    
    printf("  After:  ");
    for (int i = 0; i < count2; i++) {
        printf("%d ", nums2[i]);
    }
    printf("\n");
    
    /* Test 3: Reverse sorted array */
    printf("\nTest 3: Reverse sorted array\n");
    int nums3[] = {9, 8, 7, 6, 5, 4, 3, 2, 1};
    int count3 = sizeof(nums3) / sizeof(nums3[0]);
    
    printf("  Before: ");
    for (int i = 0; i < count3; i++) {
        printf("%d ", nums3[i]);
    }
    printf("\n");
    
    qsort(nums3, count3, sizeof(int), compare_ints);
    
    printf("  After:  ");
    for (int i = 0; i < count3; i++) {
        printf("%d ", nums3[i]);
    }
    printf("\n");
    
    /* Test 4: Single element */
    printf("\nTest 4: Single element\n");
    int nums4[] = {42};
    int count4 = 1;
    
    printf("  Before: %d\n", nums4[0]);
    qsort(nums4, count4, sizeof(int), compare_ints);
    printf("  After:  %d\n", nums4[0]);
    
    /* Test 5: Two elements */
    printf("\nTest 5: Two elements\n");
    int nums5[] = {20, 10};
    int count5 = 2;
    
    printf("  Before: ");
    for (int i = 0; i < count5; i++) {
        printf("%d ", nums5[i]);
    }
    printf("\n");
    
    qsort(nums5, count5, sizeof(int), compare_ints);
    
    printf("  After:  ");
    for (int i = 0; i < count5; i++) {
        printf("%d ", nums5[i]);
    }
    printf("\n");
    
    /* Test 6: Duplicate elements */
    printf("\nTest 6: Array with duplicates\n");
    int nums6[] = {3, 1, 4, 1, 5, 9, 2, 6, 5, 3};
    int count6 = sizeof(nums6) / sizeof(nums6[0]);
    
    printf("  Before: ");
    for (int i = 0; i < count6; i++) {
        printf("%d ", nums6[i]);
    }
    printf("\n");
    
    qsort(nums6, count6, sizeof(int), compare_ints);
    
    printf("  After:  ");
    for (int i = 0; i < count6; i++) {
        printf("%d ", nums6[i]);
    }
    printf("\n");
    
    printf("\n=== QSort Test Complete ===\n");
    printf("If all tests show properly sorted arrays, qsort is working!\n");
    
    return 0;
}