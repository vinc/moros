#include <sys/stat.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include "syscall.h"

/* MOROS FileInfo structure - matches moros/src/sys/fs/mod.rs */
struct moros_fileinfo {
    unsigned char kind;      /* File type */
    unsigned char reserved[3];
    unsigned int size;       /* File size */
    unsigned int time;       /* Timestamp */
    unsigned char name_len;  /* Filename length */
    char name[256];          /* Filename */
};

/* Convert MOROS file type to stat mode */
static mode_t moros_kind_to_mode(unsigned char kind) {
    switch (kind) {
        case 0x01: /* Directory */
            return S_IFDIR | S_IRUSR | S_IWUSR | S_IXUSR | S_IRGRP | S_IXGRP | S_IROTH | S_IXOTH;
        case 0x02: /* Device */
            return S_IFCHR | S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH | S_IWOTH;
        case 0x00: /* Regular file */
        default:
            return S_IFREG | S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
    }
}

/* Get file status */
int stat(const char* pathname, struct stat* buf) {
    if (!pathname || !buf) {
        errno = EINVAL;
        return -1;
    }
    
    /* Clear the stat buffer */
    memset(buf, 0, sizeof(struct stat));
    
    /* Use SYS_INFO to get file information */
    struct moros_fileinfo info;
    memset(&info, 0, sizeof(info));
    
    /* MOROS sys_info returns 0 on success, -1 on failure */
    long result = sys_info(pathname, &info);
    if (result < 0) {
        errno = ENOENT;
        return -1;
    }
    
    /* Fill in the stat structure with MOROS file info */
    buf->st_dev = 1;                    /* Device ID */
    buf->st_ino = 1;                    /* Inode number (simplified) */
    buf->st_nlink = 1;                  /* Number of hard links */
    buf->st_uid = 0;                    /* User ID */
    buf->st_gid = 0;                    /* Group ID */
    buf->st_rdev = 0;                   /* Device ID (if special file) */
    buf->st_size = info.size;           /* File size from MOROS */
    buf->st_atime = info.time;          /* Access time */
    buf->st_mtime = info.time;          /* Modification time */
    buf->st_ctime = info.time;          /* Status change time */
    buf->st_mode = moros_kind_to_mode(info.kind); /* File type and permissions */
    
    return 0;
}

/* Get file status from file descriptor */
int fstat(int fd, struct stat* buf) {
    if (!buf) {
        errno = EINVAL;
        return -1;
    }
    
    /* Check if file descriptor is valid */
    if (fd < 0) {
        errno = EBADF;
        return -1;
    }
    
    /* Clear the stat buffer */
    memset(buf, 0, sizeof(struct stat));
    
    /* For MOROS, we can't easily get file info from just a file descriptor */
    /* We'll provide basic information based on the fd */
    buf->st_dev = 1;        /* Device ID */
    buf->st_ino = fd;       /* Use fd as inode for simplicity */
    buf->st_nlink = 1;      /* Number of hard links */
    buf->st_uid = 0;        /* User ID */
    buf->st_gid = 0;        /* Group ID */
    buf->st_rdev = 0;       /* Device ID (if special file) */
    buf->st_size = 0;       /* File size - unknown from fd alone */
    buf->st_atime = 0;      /* Access time */
    buf->st_mtime = 0;      /* Modification time */
    buf->st_ctime = 0;      /* Status change time */
    
    /* For standard streams, mark as character devices */
    if (fd <= 2) {
        buf->st_mode = S_IFCHR | S_IRUSR | S_IWUSR;
    } else {
        buf->st_mode = S_IFREG | S_IRUSR | S_IWUSR | S_IRGRP | S_IROTH;
    }
    
    return 0;
}

/* Get file status (same as stat for now) */
int lstat(const char* pathname, struct stat* buf) {
    /* In a full implementation, lstat wouldn't follow symbolic links */
    /* For MOROS, just use stat for now since we don't have symlinks */
    return stat(pathname, buf);
}

/* Create directory */
int mkdir(const char* pathname, mode_t mode) {
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    
    /* Try to create directory using sys_open with create and dir flags */
    /* OpenFlag::Create | OpenFlag::Dir = 0x10 | 0x04 = 0x14 */
    long handle = sys_open(pathname, 0x14);
    if (handle < 0) {
        errno = EEXIST; /* Assume it already exists */
        return -1;
    }
    
    /* Close the directory handle */
    sys_close((int)handle);
    return 0;
}

/* Change file permissions */
int chmod(const char* pathname, mode_t mode) {
    if (!pathname) {
        errno = EINVAL;
        return -1;
    }
    
    /* MOROS doesn't have chmod syscall yet */
    /* For now, just check if file exists using stat */
    struct stat st;
    if (stat(pathname, &st) != 0) {
        return -1; /* errno already set by stat */
    }
    
    /* Pretend success for now */
    return 0;
}

/* Change file permissions via file descriptor */
int fchmod(int fd, mode_t mode) {
    if (fd < 0) {
        errno = EBADF;
        return -1;
    }
    
    /* MOROS doesn't have fchmod syscall yet */
    /* Pretend success for now */
    return 0;
}

/* Set file mode creation mask */
mode_t umask(mode_t mask) {
    /* MOROS doesn't have umask concept yet */
    /* Return previous mask (assume 022) */
    static mode_t current_mask = 022;
    mode_t old_mask = current_mask;
    current_mask = mask & 0777;
    return old_mask;
}