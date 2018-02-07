#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

extern void pacman_init(void *(*add)(const char *));

extern void pacman_update(void);

extern void pacman_render(void);

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H
