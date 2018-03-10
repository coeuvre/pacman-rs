#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

enum PlatformEventKind {
    PLATFORM_EVENT_CLOSE = 1,
};

typedef struct PlatformEvent {
    int kind;
} PlatformEvent;

typedef struct PlatformApi {
    int (*poll_event)(PlatformEvent *event);

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
