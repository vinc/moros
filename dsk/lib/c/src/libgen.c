#include <libgen.h>
#include <string.h>
#include <stdlib.h>

/* POSIX-compliant dirname and basename that modify the input string
 * This avoids static buffer issues and is safer for MOROS memory layout */

/* Extract filename from path - modifies input string */
char* basename(char* path) {
    char* p;
    int len;
    
    /* Handle NULL or empty string */
    if (!path || *path == '\0') {
        return ".";
    }
    
    /* Get string length */
    len = strlen(path);
    
    /* Handle root directory case */
    if (len == 1 && path[0] == '/') {
        return path;
    }
    
    /* Remove trailing slashes (except if it's just "/") */
    while (len > 1 && path[len - 1] == '/') {
        path[len - 1] = '\0';
        len--;
    }
    
    /* If we removed all characters, it was all slashes */
    if (len == 0 || path[0] == '\0') {
        path[0] = '/';
        path[1] = '\0';
        return path;
    }
    
    /* Find the last slash */
    p = NULL;
    char* current = path;
    while (*current) {
        if (*current == '/') {
            p = current;
        }
        current++;
    }
    
    /* If we found a slash, return everything after the last one */
    if (p && *(p + 1) != '\0') {
        return p + 1;
    }
    
    /* No slash found, return the whole string */
    return path;
}

/* Extract directory from path - modifies input string */
char* dirname(char* path) {
    char* p;
    int len;
    char* last_slash;
    
    /* Handle NULL or empty string */
    if (!path || *path == '\0') {
        return ".";
    }
    
    /* Get string length */
    len = strlen(path);
    
    /* Handle root directory case */
    if (len == 1) {
        if (path[0] == '/') {
            return path;  /* Return "/" */
        } else {
            return ".";   /* Single character, return current dir */
        }
    }
    
    /* Remove trailing slashes (except if it's just "/") */
    while (len > 1 && path[len - 1] == '/') {
        path[len - 1] = '\0';
        len--;
    }
    
    /* If we removed all characters, it was all slashes */
    if (len == 0 || path[0] == '\0') {
        path[0] = '/';
        path[1] = '\0';
        return path;
    }
    
    /* Find the last slash */
    last_slash = NULL;
    p = path;
    while (*p) {
        if (*p == '/') {
            last_slash = p;
        }
        p++;
    }
    
    /* No slash found */
    if (!last_slash) {
        return ".";
    }
    
    /* If slash is at the beginning, return "/" */
    if (last_slash == path) {
        path[1] = '\0';
        return path;
    }
    
    /* Terminate string at the last slash */
    *last_slash = '\0';
    
    /* Remove any trailing slashes from the result */
    p = last_slash - 1;
    while (p > path && *p == '/') {
        *p = '\0';
        p--;
    }
    
    /* If we ended up with empty string, return "/" */
    if (path[0] == '\0') {
        path[0] = '/';
        path[1] = '\0';
    }
    
    return path;
}