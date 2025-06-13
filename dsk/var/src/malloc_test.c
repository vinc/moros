#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char* argv[]) {
    printf("MOROS Memory Allocation Test\n");
    printf("============================\n\n");
    
    // Test basic malloc
    printf("Testing malloc...\n");
    char* buffer = malloc(100);
    if (buffer) {
        strcpy(buffer, "Hello from malloc!");
        printf("Success: %s\n", buffer);
        free(buffer);
        printf("Memory freed\n\n");
    } else {
        printf("Failed to allocate memory\n\n");
        return 1;
    }
    
    // Test calloc
    printf("Testing calloc...\n");
    int* numbers = calloc(10, sizeof(int));
    if (numbers) {
        printf("Calloc successful, initial values: ");
        for (int i = 0; i < 10; i++) {
            printf("%d ", numbers[i]);
        }
        printf("\n");
        
        // Fill with data
        for (int i = 0; i < 10; i++) {
            numbers[i] = i * i;
        }
        
        printf("After filling with squares: ");
        for (int i = 0; i < 10; i++) {
            printf("%d ", numbers[i]);
        }
        printf("\n");
        
        free(numbers);
        printf("Calloc memory freed\n\n");
    } else {
        printf("Failed to allocate memory with calloc\n\n");
    }
    
    // Test realloc
    printf("Testing realloc...\n");
    char* dynamic_buffer = malloc(50);
    if (dynamic_buffer) {
        strcpy(dynamic_buffer, "Initial string");
        printf("Initial: %s\n", dynamic_buffer);
        
        // Expand the buffer
        dynamic_buffer = realloc(dynamic_buffer, 100);
        if (dynamic_buffer) {
            strcat(dynamic_buffer, " - expanded with realloc!");
            printf("After realloc: %s\n", dynamic_buffer);
            free(dynamic_buffer);
            printf("Realloc memory freed\n\n");
        } else {
            printf("Realloc failed\n\n");
        }
    } else {
        printf("Initial malloc for realloc test failed\n\n");
    }
    
    // Test multiple allocations
    printf("Testing multiple allocations...\n");
    void* ptrs[5];
    int sizes[] = {16, 32, 64, 128, 256};
    
    for (int i = 0; i < 5; i++) {
        ptrs[i] = malloc(sizes[i]);
        if (ptrs[i]) {
            printf("Allocated %d bytes at %p\n", sizes[i], ptrs[i]);
        } else {
            printf("Failed to allocate %d bytes\n", sizes[i]);
        }
    }
    
    // Free all allocations
    printf("Freeing all allocations...\n");
    for (int i = 0; i < 5; i++) {
        if (ptrs[i]) {
            free(ptrs[i]);
            printf("Freed pointer %d\n", i);
        }
    }
    
    printf("\nAll memory allocation tests completed!\n");
    return 0;
}