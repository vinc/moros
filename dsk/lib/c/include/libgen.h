#ifndef _LIBGEN_H
#define _LIBGEN_H

#ifdef __cplusplus
extern "C" {
#endif

/* Path manipulation functions */
char* basename(char* path);
char* dirname(char* path);

#ifdef __cplusplus
}
#endif

#endif /* _LIBGEN_H */