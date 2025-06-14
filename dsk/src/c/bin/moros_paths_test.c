#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/stat.h>
#include <dirent.h>
#include <errno.h>
#include <libgen.h>

int main(void) {
    printf("=== MOROS Paths Test ===\n");
    printf("Testing with actual MOROS filesystem paths...\n\n");
    
    /* Test 1: Check /ini directory (should exist) */
    printf("1. Testing /ini directory:\n");
    if (access("/ini", F_OK) == 0) {
        printf("   ✓ access(\"/ini\", F_OK) - ini directory exists\n");
        
        struct stat st;
        if (stat("/ini", &st) == 0) {
            printf("   ✓ stat(\"/ini\") successful\n");
            if (S_ISDIR(st.st_mode)) {
                printf("   ✓ /ini is a directory\n");
            }
        } else {
            printf("   ✗ stat(\"/ini\") failed\n");
        }
        
        /* Try to open directory */
        DIR* dir = opendir("/ini");
        if (dir) {
            printf("   ✓ opendir(\"/ini\") successful\n");
            
            struct dirent* entry;
            int count = 0;
            printf("   Directory contents:\n");
            while ((entry = readdir(dir)) != NULL && count < 10) {
                printf("     - %s\n", entry->d_name);
                count++;
            }
            
            if (count == 0) {
                printf("     (empty or readdir not working)\n");
            } else {
                printf("     Found %d entries\n", count);
            }
            
            closedir(dir);
            printf("   ✓ closedir() successful\n");
        } else {
            printf("   ✗ opendir(\"/ini\") failed (errno: %d)\n", errno);
        }
    } else {
        printf("   ✗ access(\"/ini\", F_OK) failed (errno: %d)\n", errno);
    }
    
    /* Test 2: Test device files */
    printf("\n2. Testing device files:\n");
    const char* devices[] = {
        "/dev/clk/epoch",
        "/dev/clk/boot", 
        "/dev/random",
        "/dev/null"
    };
    
    for (int i = 0; i < 4; i++) {
        if (access(devices[i], F_OK) == 0) {
            printf("   ✓ %s exists\n", devices[i]);
        } else {
            printf("   ✗ %s not found\n", devices[i]);
        }
    }
    
    /* Test 3: Path manipulation with MOROS paths */
    printf("\n3. Testing path manipulation:\n");
    
    /* Test with /ini/test path */
    char test_path1[] = "/ini/test";
    printf("   Original: %s\n", test_path1);
    char* dir1 = dirname(test_path1);
    printf("   dirname:  '%s'\n", dir1 ? dir1 : "NULL");
    
    /* Reset and test basename */
    strcpy(test_path1, "/ini/test");
    char* base1 = basename(test_path1);
    printf("   basename: '%s'\n", base1 ? base1 : "NULL");
    
    /* Test with device path */
    char test_path2[] = "/dev/clk/epoch";
    printf("   Original: %s\n", test_path2);
    char* dir2 = dirname(test_path2);
    printf("   dirname:  '%s'\n", dir2 ? dir2 : "NULL");
    
    strcpy(test_path2, "/dev/clk/epoch");
    char* base2 = basename(test_path2);
    printf("   basename: '%s'\n", base2 ? base2 : "NULL");
    
    /* Test 4: Try to read from a device file */
    printf("\n4. Testing device file reading:\n");
    FILE* epoch_file = fopen("/dev/clk/epoch", "r");
    if (epoch_file) {
        printf("   ✓ fopen(\"/dev/clk/epoch\", \"r\") successful\n");
        
        char buffer[64];
        if (fgets(buffer, sizeof(buffer), epoch_file)) {
            printf("   Read from epoch: %s", buffer);
        } else {
            printf("   Could not read from epoch file\n");
        }
        
        fclose(epoch_file);
        printf("   ✓ fclose() successful\n");
    } else {
        printf("   ✗ fopen(\"/dev/clk/epoch\", \"r\") failed\n");
    }
    
    /* Test 5: Check root directory */
    printf("\n5. Testing root directory:\n");
    DIR* root_dir = opendir("/");
    if (root_dir) {
        printf("   ✓ opendir(\"/\") successful\n");
        
        struct dirent* entry;
        int count = 0;
        printf("   Root directory contents:\n");
        while ((entry = readdir(root_dir)) != NULL && count < 20) {
            printf("     - %s (type: %d)\n", entry->d_name, entry->d_type);
            count++;
        }
        
        printf("   Found %d entries in root\n", count);
        closedir(root_dir);
    } else {
        printf("   ✗ opendir(\"/\") failed (errno: %d)\n", errno);
    }
    
    /* Test 6: Check if we can access current working directory */
    printf("\n6. Testing current directory:\n");
    char cwd_buffer[256];
    char* cwd = getcwd(cwd_buffer, sizeof(cwd_buffer));
    if (cwd) {
        printf("   Current directory: %s\n", cwd);
        
        if (access(cwd, F_OK) == 0) {
            printf("   ✓ Current directory accessible\n");
        } else {
            printf("   ✗ Current directory not accessible\n");
        }
    } else {
        printf("   ✗ getcwd() failed\n");
    }
    
    printf("\n=== MOROS Paths Test Complete ===\n");
    printf("This test helps identify which MOROS filesystem features work.\n");
    
    return 0;
}