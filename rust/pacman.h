#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

enum PlatformLogLevel {
    PLATFORM_LOG_LEVEL_ERROR = 1,
    PLATFORM_LOG_LEVEL_WARN,
    PLATFORM_LOG_LEVEL_INFO,
    PLATFORM_LOG_LEVEL_DEBUG,
    PLATFORM_LOG_LEVEL_TRACE,
};

typedef struct PlatformApi {
    void (*quit)(void);
    void (*log)(int level, const char *message);
    void *(*get_gl_proc_address)(const char *name);
    float (*get_delta_time)(void);
} PlatformApi;

enum PlatformEventId {
    PLATFORM_EVENT_CLOSE,
};

typedef struct LibApi {
    void (*on_platform_event)(int event_id, void *data);
    void (*update)(void);
    void (*render)(void);
} LibApi;

extern LibApi *pacman_load(PlatformApi *);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
