#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

enum PlatformEventId {
    PLATFORM_EVENT_CLOSE,
};;

typedef struct PlatformApi {
    void (*log)(const char *message);
    void *(*get_gl_proc_address)(const char *name);
    void (*swap_gl_buffer)(void);
    uint64_t (*get_performance_counter)(void);
    uint64_t(*get_performance_frequency)(void);
} PlatformApi;

extern void pacman_start(PlatformApi *);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
