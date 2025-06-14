#ifndef _STDDEF_H
#define _STDDEF_H

#ifdef __cplusplus
extern "C" {
#endif

/* Basic types */
typedef unsigned long size_t;
typedef long ptrdiff_t;
typedef long ssize_t;
typedef long off_t;

#ifndef NULL
#define NULL ((void*)0)
#endif

/* offsetof macro */
#define offsetof(type, member) __builtin_offsetof(type, member)

#ifdef __cplusplus
}
#endif

#endif /* _STDDEF_H */