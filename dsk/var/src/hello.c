#include <stdio.h>
#include <stdlib.h>

int main(int argc, char* argv[]) {
    printf("Hello, World from C!\n");
    printf("This is a C program running on MOROS\n");
    
    if (argc > 1) {
        printf("Command line arguments:\n");
        for (int i = 0; i < argc; i++) {
            printf("  argv[%d] = %s\n", i, argv[i]);
        }
    }
    
    return 0;
}