#ifndef __BIRD_H
#define __BIRD_H

#include <Q9_6.h>

#define BIRD_WIDTH 6
#define BIRD_HEIGHT 6
#define BIRD_OBMAS_SIZE 1
#define BIRD_FLAP_STRENGTH -(SINT_TO_Q9_6(2))
#define BIRD_GRAVITY ((Q9_6)0x000a)
#define BIRD_X 48

typedef struct bird_s {
    Q9_6 y_velocity, y;
    uint8_t obmas[BIRD_OBMAS_SIZE];
} bird_t;

void bird_init(bird_t *);

void bird_flap(bird_t *);

void bird_move(bird_t *);

void bird_draw(bird_t *);

#endif