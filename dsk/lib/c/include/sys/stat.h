#ifndef _SYS_STAT_H
#define _SYS_STAT_H

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/* File types */
#define S_IFMT   0170000  /* File type mask */
#define S_IFREG  0100000  /* Regular file */
#define S_IFDIR  0040000  /* Directory */
#define S_IFCHR  0020000  /* Character device */
#define S_IFBLK  0060000  /* Block device */
#define S_IFIFO  0010000  /* FIFO */
#define S_IFLNK  0120000  /* Symbolic link */
#define S_IFSOCK 0140000  /* Socket */

/* File permissions */
#define S_IRWXU 0700      /* Owner: read, write, execute */
#define S_IRUSR 0400      /* Owner: read */
#define S_IWUSR 0200      /* Owner: write */
#define S_IXUSR 0100      /* Owner: execute */

#define S_IRWXG 0070      /* Group: read, write, execute */
#define S_IRGRP 0040      /* Group: read */
#define S_IWGRP 0020      /* Group: write */
#define S_IXGRP 0010      /* Group: execute */

#define S_IRWXO 0007      /* Others: read, write, execute */
#define S_IROTH 0004      /* Others: read */
#define S_IWOTH 0002      /* Others: write */
#define S_IXOTH 0001      /* Others: execute */

/* File type test macros */
#define S_ISREG(m)  (((m) & S_IFMT) == S_IFREG)   /* Regular file */
#define S_ISDIR(m)  (((m) & S_IFMT) == S_IFDIR)   /* Directory */
#define S_ISCHR(m)  (((m) & S_IFMT) == S_IFCHR)   /* Character device */
#define S_ISBLK(m)  (((m) & S_IFMT) == S_IFBLK)   /* Block device */
#define S_ISFIFO(m) (((m) & S_IFMT) == S_IFIFO)   /* FIFO */
#define S_ISLNK(m)  (((m) & S_IFMT) == S_IFLNK)   /* Symbolic link */
#define S_ISSOCK(m) (((m) & S_IFMT) == S_IFSOCK)  /* Socket */

/* Type definitions */
typedef unsigned int mode_t;
typedef unsigned int uid_t;
typedef unsigned int gid_t;
typedef unsigned long ino_t;
typedef unsigned long dev_t;
typedef unsigned int nlink_t;
#ifndef _TIME_T_DEFINED
#define _TIME_T_DEFINED
typedef long time_t;
#endif

/* File status structure */
struct stat {
    dev_t     st_dev;     /* Device ID */
    ino_t     st_ino;     /* Inode number */
    mode_t    st_mode;    /* File type and permissions */
    nlink_t   st_nlink;   /* Number of hard links */
    uid_t     st_uid;     /* User ID of owner */
    gid_t     st_gid;     /* Group ID of owner */
    dev_t     st_rdev;    /* Device ID (if special file) */
    off_t     st_size;    /* Total size in bytes */
    time_t    st_atime;   /* Last access time */
    time_t    st_mtime;   /* Last modification time */
    time_t    st_ctime;   /* Last status change time */
};

/* Function declarations */
int stat(const char* pathname, struct stat* buf);
int fstat(int fd, struct stat* buf);
int lstat(const char* pathname, struct stat* buf);

int mkdir(const char* pathname, mode_t mode);
int chmod(const char* pathname, mode_t mode);
int fchmod(int fd, mode_t mode);

mode_t umask(mode_t mask);

#ifdef __cplusplus
}
#endif

#endif /* _SYS_STAT_H */