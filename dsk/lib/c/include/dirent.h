#ifndef _DIRENT_H
#define _DIRENT_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Directory entry types */
#define DT_UNKNOWN 0
#define DT_FIFO    1
#define DT_CHR     2
#define DT_DIR     4
#define DT_BLK     6
#define DT_REG     8
#define DT_LNK    10
#define DT_SOCK   12

/* Maximum filename length */
#define NAME_MAX 255

/* Directory entry structure */
struct dirent {
    unsigned long d_ino;           /* Inode number */
    unsigned short d_reclen;       /* Length of this record */
    unsigned char d_type;          /* File type */
    char d_name[NAME_MAX + 1];     /* Null-terminated filename */
};

/* Directory stream type */
typedef struct {
    int fd;                        /* File descriptor */
    char* buffer;                  /* Buffer for directory entries */
    size_t buffer_size;            /* Size of buffer */
    size_t buffer_pos;             /* Current position in buffer */
    size_t buffer_end;             /* End of valid data in buffer */
    struct dirent entry;           /* Current directory entry */
} DIR;

/* Function declarations */
DIR* opendir(const char* name);
struct dirent* readdir(DIR* dirp);
int readdir_r(DIR* dirp, struct dirent* entry, struct dirent** result);
void rewinddir(DIR* dirp);
long telldir(DIR* dirp);
void seekdir(DIR* dirp, long loc);
int closedir(DIR* dirp);

/* Directory file descriptor operations */
int dirfd(DIR* dirp);
DIR* fdopendir(int fd);

/* Scanning functions */
int scandir(const char* dirp, struct dirent*** namelist,
            int (*filter)(const struct dirent*),
            int (*compar)(const struct dirent**, const struct dirent**));
int alphasort(const struct dirent** a, const struct dirent** b);
int versionsort(const struct dirent** a, const struct dirent** b);

#ifdef __cplusplus
}
#endif

#endif /* _DIRENT_H */