#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

typedef void PacManMakeGLContextCurrent(void *context);
typedef void *PacManGetGLProcAddress(char *name);

extern void pacman_init(PacManGetGLProcAddress *);

extern void pacman_update(void);

extern void pacman_render(void);

extern void pacman_start(void *context, PacManMakeGLContextCurrent *, PacManMakeGLContextCurrent *);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
