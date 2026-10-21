// Flattens CASC_FIND_DATA so Rust does not depend on its struct layout.
#include <string.h>

#include "CascLib.h"

// Bounded copy with an explicit length: portable, always NUL-terminated, and
// free of strncpy, which MSVC rejects as deprecated (C4996).
static void copy_found(const CASC_FIND_DATA &fd, char *name, size_t cap, unsigned long long *size)
{
    size_t length = strlen(fd.szFileName);

    if (length >= cap) {
        length = cap - 1;
    }

    memcpy(name, fd.szFileName, length);
    name[length] = 0;
    *size = fd.FileSize;
}

extern "C" HANDLE casc_shim_find_first(HANDLE storage, const char *mask, char *name, size_t cap,
                                       unsigned long long *size)
{
    CASC_FIND_DATA fd;
    HANDLE find = CascFindFirstFile(storage, mask, &fd, NULL);

    if (find != NULL && find != INVALID_HANDLE_VALUE) {
        copy_found(fd, name, cap, size);
    }

    return find;
}

extern "C" bool casc_shim_find_next(HANDLE find, char *name, size_t cap, unsigned long long *size)
{
    CASC_FIND_DATA fd;

    if (!CascFindNextFile(find, &fd)) {
        return false;
    }

    copy_found(fd, name, cap, size);
    return true;
}
