#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Platform {
    void *(*get_gl_proc_address)(const char *name);
} Platform;

typedef struct PacManLib {
    void (*on_platform_event)(int event_id, void *data);
    void (*update)(void);
    void (*render)(void);
} PacManLib;

extern PacManLib *pacman_init(Platform *);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
