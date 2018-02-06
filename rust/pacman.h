#ifndef PACMAN_H
#define PACMAN_H

#ifdef __cplusplus
extern "C" {
#endif

void pacman_init(void *(*get_proc_address)(const char *name));

void pacman_update();

void pacman_render();

#ifdef __cplusplus
}
#endif

#endif // PACMAN_H