#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Platform {
    void *(*get_gl_proc_address)(const char *name);
} Platform;

typedef struct PacManLib {
    void (*on_platform_event)(Platform *, int event_id, void *data);
    void (*update)(Platform *);
    void (*render)(Platform *);
} PacManLib;

extern PacManLib *pacman_load(Platform *);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
