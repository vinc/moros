#include <dirent.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <unistd.h>
#include "syscall.h"

/* Open directory stream */
DIR* opendir(const char* name) {
    if (!name) {
        errno = EINVAL;
        return NULL;
    }
    
    /* For MOROS, we need to open directories with OpenFlag::Dir (0x20) */
    long handle = sys_open(name, 0x20); /* OpenFlag::Dir */
    if (handle < 0) {
        errno = ENOENT;
        return NULL;
    }
    
    /* Allocate DIR structure */
    DIR* dir = malloc(sizeof(DIR));
    if (!dir) {
        sys_close((int)handle);
        errno = ENOMEM;
        return NULL;
    }
    
    /* Initialize DIR structure */
    dir->fd = (int)handle;
    dir->buffer_size = 4096;  /* Larger buffer for directory contents */
    dir->buffer = malloc(dir->buffer_size);
    if (!dir->buffer) {
        sys_close((int)handle);
        free(dir);
        errno = ENOMEM;
        return NULL;
    }
    
    /* Read directory contents into buffer */
    long bytes_read = sys_read(dir->fd, dir->buffer, dir->buffer_size);
    if (bytes_read < 0) {
        sys_close(dir->fd);
        free(dir->buffer);
        free(dir);
        errno = EIO;
        return NULL;
    }
    
    dir->buffer_pos = 0;
    dir->buffer_end = (size_t)bytes_read;
    memset(&dir->entry, 0, sizeof(struct dirent));
    
    return dir;
}

/* Read directory entry */
struct dirent* readdir(DIR* dirp) {
    if (!dirp) {
        errno = EINVAL;
        return NULL;
    }
    
    /* Check if we've reached the end of the buffer */
    if (dirp->buffer_pos >= dirp->buffer_end) {
        return NULL;
    }
    
    /* Parse MOROS directory entry format */
    /* Format: 14 bytes of metadata + filename length (at offset 13) + filename */
    unsigned char* buf = (unsigned char*)dirp->buffer;
    size_t pos = dirp->buffer_pos;
    
    /* Check if we have enough bytes for the header */
    if (pos + 14 > dirp->buffer_end) {
        return NULL;
    }
    
    /* Get filename length from offset 13 */
    unsigned char filename_len = buf[pos + 13];
    
    /* Sanity check filename length */
    if (filename_len == 0 || filename_len > NAME_MAX) {
        return NULL;
    }
    
    /* Calculate total entry size */
    size_t entry_size = 14 + filename_len;
    
    /* Check if we have enough bytes for the complete entry */
    if (pos + entry_size > dirp->buffer_end) {
        return NULL;
    }
    
    /* Fill in the dirent structure */
    dirp->entry.d_ino = pos + 1; /* Use position as inode for simplicity */
    dirp->entry.d_reclen = entry_size;
    
    /* Parse file type from metadata (first byte contains type info) */
    unsigned char type_byte = buf[pos];
    if (type_byte & 0x01) {
        dirp->entry.d_type = DT_DIR;
    } else if (type_byte & 0x02) {
        dirp->entry.d_type = DT_CHR; /* Device file */
    } else {
        dirp->entry.d_type = DT_REG; /* Regular file */
    }
    
    /* Copy filename (ensure null termination) */
    size_t copy_len = (filename_len < NAME_MAX) ? filename_len : NAME_MAX - 1;
    memcpy(dirp->entry.d_name, &buf[pos + 14], copy_len);
    dirp->entry.d_name[copy_len] = '\0';
    
    /* Advance buffer position */
    dirp->buffer_pos = pos + entry_size;
    
    return &dirp->entry;
}

/* Thread-safe version of readdir */
int readdir_r(DIR* dirp, struct dirent* entry, struct dirent** result) {
    if (!dirp || !entry || !result) {
        return EINVAL;
    }
    
    *result = readdir(dirp);
    if (*result) {
        memcpy(entry, *result, sizeof(struct dirent));
        *result = entry;
        return 0;
    }
    
    return 0; /* End of directory */
}

/* Rewind directory stream */
void rewinddir(DIR* dirp) {
    if (!dirp) {
        return;
    }
    
    dirp->buffer_pos = 0;
}

/* Get current position in directory stream */
long telldir(DIR* dirp) {
    if (!dirp) {
        errno = EINVAL;
        return -1;
    }
    
    return (long)dirp->buffer_pos;
}

/* Seek to position in directory stream */
void seekdir(DIR* dirp, long loc) {
    if (!dirp) {
        return;
    }
    
    if (loc >= 0 && (size_t)loc <= dirp->buffer_end) {
        dirp->buffer_pos = (size_t)loc;
    }
}

/* Close directory stream */
int closedir(DIR* dirp) {
    if (!dirp) {
        errno = EINVAL;
        return -1;
    }
    
    if (dirp->buffer) {
        free(dirp->buffer);
    }
    
    if (dirp->fd >= 0) {
        close(dirp->fd);
    }
    
    free(dirp);
    return 0;
}

/* Get file descriptor from directory stream */
int dirfd(DIR* dirp) {
    if (!dirp) {
        errno = EINVAL;
        return -1;
    }
    
    return dirp->fd;
}

/* Create directory stream from file descriptor */
DIR* fdopendir(int fd) {
    if (fd < 0) {
        errno = EBADF;
        return NULL;
    }
    
    /* Allocate DIR structure */
    DIR* dir = malloc(sizeof(DIR));
    if (!dir) {
        errno = ENOMEM;
        return NULL;
    }
    
    /* Initialize DIR structure */
    dir->fd = fd;
    dir->buffer_size = 4096;
    dir->buffer = malloc(dir->buffer_size);
    if (!dir->buffer) {
        free(dir);
        errno = ENOMEM;
        return NULL;
    }
    
    /* Read directory contents */
    long bytes_read = sys_read(fd, dir->buffer, dir->buffer_size);
    if (bytes_read < 0) {
        free(dir->buffer);
        free(dir);
        errno = EIO;
        return NULL;
    }
    
    dir->buffer_pos = 0;
    dir->buffer_end = (size_t)bytes_read;
    memset(&dir->entry, 0, sizeof(struct dirent));
    
    return dir;
}

/* Scan directory for entries matching filter */
int scandir(const char* dirp, struct dirent*** namelist,
            int (*filter)(const struct dirent*),
            int (*compar)(const struct dirent**, const struct dirent**)) {
    if (!dirp || !namelist) {
        errno = EINVAL;
        return -1;
    }
    
    DIR* dir = opendir(dirp);
    if (!dir) {
        return -1;
    }
    
    /* Count entries first */
    struct dirent* entry;
    int count = 0;
    
    while ((entry = readdir(dir)) != NULL) {
        if (!filter || filter(entry)) {
            count++;
        }
    }
    
    if (count == 0) {
        closedir(dir);
        *namelist = NULL;
        return 0;
    }
    
    /* Allocate array for results */
    struct dirent** list = malloc(count * sizeof(struct dirent*));
    if (!list) {
        closedir(dir);
        errno = ENOMEM;
        return -1;
    }
    
    /* Read entries again */
    rewinddir(dir);
    int i = 0;
    
    while ((entry = readdir(dir)) != NULL && i < count) {
        if (!filter || filter(entry)) {
            list[i] = malloc(sizeof(struct dirent));
            if (!list[i]) {
                /* Cleanup on error */
                for (int j = 0; j < i; j++) {
                    free(list[j]);
                }
                free(list);
                closedir(dir);
                errno = ENOMEM;
                return -1;
            }
            memcpy(list[i], entry, sizeof(struct dirent));
            i++;
        }
    }
    
    closedir(dir);
    
    /* Sort if comparator provided */
    if (compar) {
        /* Simple bubble sort for small directories */
        for (int i = 0; i < count - 1; i++) {
            for (int j = 0; j < count - i - 1; j++) {
                if (compar((const struct dirent**)&list[j], (const struct dirent**)&list[j + 1]) > 0) {
                    struct dirent* temp = list[j];
                    list[j] = list[j + 1];
                    list[j + 1] = temp;
                }
            }
        }
    }
    
    *namelist = list;
    return count;
}

/* Alphabetical comparison for directory entries */
int alphasort(const struct dirent** a, const struct dirent** b) {
    if (!a || !b || !*a || !*b) {
        return 0;
    }
    
    return strcmp((*a)->d_name, (*b)->d_name);
}

/* Version comparison for directory entries */
int versionsort(const struct dirent** a, const struct dirent** b) {
    /* For now, just use alphabetical sort */
    return alphasort(a, b);
}