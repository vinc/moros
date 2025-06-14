#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char* argv[]) {
    printf("MOROS libc Test Program\n");
    printf("=======================\n\n");
    
    // Test basic I/O
    printf("Testing basic I/O:\n");
    printf("Hello, World! (printf)\n");
    puts("Hello, World! (puts)");
    
    // Test string functions
    printf("\nTesting string functions:\n");
    char str1[100] = "Hello";
    char str2[100] = " World";
    
    printf("str1 = \"%s\", length = %d\n", str1, (int)strlen(str1));
    printf("str2 = \"%s\", length = %d\n", str2, (int)strlen(str2));
    
    strcat(str1, str2);
    printf("After strcat: \"%s\", length = %d\n", str1, (int)strlen(str1));
    
    // Test string comparison
    char str3[] = "Hello World";
    printf("strcmp result: %d\n", strcmp(str1, str3));
    
    // Test memory functions
    printf("\nTesting memory functions:\n");
    char buffer[20];
    memset(buffer, 'A', 10);
    buffer[10] = '\0';
    printf("After memset: \"%s\"\n", buffer);
    
    char src[] = "Test";
    memcpy(buffer, src, 4);
    buffer[4] = '\0';
    printf("After memcpy: \"%s\"\n", buffer);
    
    // Test memory allocation
    printf("\nTesting memory allocation:\n");
    char* dynamic_str = malloc(50);
    if (dynamic_str) {
        strcpy(dynamic_str, "Dynamically allocated string");
        printf("Dynamic string: \"%s\"\n", dynamic_str);
        free(dynamic_str);
        printf("Memory freed successfully\n");
    } else {
        printf("Memory allocation failed\n");
    }
    
    // Test calloc
    int* numbers = calloc(5, sizeof(int));
    if (numbers) {
        printf("Calloc test - initialized values: ");
        for (int i = 0; i < 5; i++) {
            printf("%d ", numbers[i]);
        }
        printf("\n");
        
        // Set some values
        for (int i = 0; i < 5; i++) {
            numbers[i] = i + 1;
        }
        
        printf("After setting values: ");
        for (int i = 0; i < 5; i++) {
            printf("%d ", numbers[i]);
        }
        printf("\n");
        
        free(numbers);
    }
    
    // Test string tokenization
    printf("\nTesting string tokenization:\n");
    char test_str[] = "apple,banana,cherry,date";
    char* token = strtok(test_str, ",");
    int count = 1;
    
    while (token != NULL) {
        printf("Token %d: \"%s\"\n", count++, token);
        token = strtok(NULL, ",");
    }
    
    // Test character functions
    printf("\nTesting character I/O:\n");
    printf("Enter a single character (press Enter): ");
    int ch = getchar();
    if (ch != EOF) {
        printf("You entered: '%c' (ASCII %d)\n", ch, ch);
    }
    
    // Test file operations (basic)
    printf("\nTesting file operations:\n");
    FILE* file = fopen("/tmp/test.txt", "w");
    if (file) {
        fprintf(file, "This is a test file.\n");
        fprintf(file, "Line 2 of the test file.\n");
        fclose(file);
        printf("Successfully wrote to file\n");
        
        // Try to read it back
        file = fopen("/tmp/test.txt", "r");
        if (file) {
            char line[100];
            printf("Reading file contents:\n");
            while (fgets(line, sizeof(line), file)) {
                printf("  %s", line);
            }
            fclose(file);
        } else {
            printf("Could not read file back\n");
        }
    } else {
        printf("Could not create test file\n");
    }
    
    // Test command line arguments
    printf("\nCommand line arguments:\n");
    printf("argc = %d\n", argc);
    for (int i = 0; i < argc; i++) {
        printf("argv[%d] = \"%s\"\n", i, argv[i]);
    }
    
    printf("\nAll tests completed successfully!\n");
    return 0;
}