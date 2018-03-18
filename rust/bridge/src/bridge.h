#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>

enum PlatformEventKind {
    PLATFORM_EVENT_RENDER = 1,
    PLATFORM_EVENT_CLOSE = 2,
    PLATFORM_EVENT_RESIZE = 3,
};

typedef struct PlatformEvent {
    int kind;
    union {
        struct {
            int width;
            int height;
        } resize;
    } data;
} PlatformEvent;

typedef struct Platform {
    void (*quit)(void);

    void *(*get_gl_proc_address)(const char *name);

    uint64_t (*get_performance_counter)(void);
    uint64_t (*get_performance_frequency)(void);
} Platform;

extern void game_load(Platform *);

extern void game_quit(void);

extern void game_on_platform_event(PlatformEvent *event);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
