#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct PlatformApi {
    void (*quit)(void);
    void (*log)(const char *message);
    void *(*get_gl_proc_address)(const char *name);
    float (*get_delta_time)(void);
} PlatformApi;

enum PlatformEventId {
    PLATFORM_EVENT_CLOSE,
};

typedef struct LibApi {
    void (*on_platform_event)(int event_id, void *data);
    void (*render)(void);
} LibApi;

extern LibApi *pacman_load(PlatformApi *);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
