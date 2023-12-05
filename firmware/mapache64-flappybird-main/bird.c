#include "bird.h"

#include <vram.h>

#include "patterns.h"
#include "pipe.h"

void bird_init(bird_t *bird) {
    uint8_t i;
    bird->y = sint16_to_Q9_6(SCREEN_START + (SCREEN_HEIGHT / 2));
    bird->y_velocity = 0;

    for (i = 0; i < BIRD_OBMAS_SIZE; i++) {
        bird->obmas[i] = i;
        OBM[i].pattern_config = bird_pattern_pmfa;
        OBM[i].color = YELLOW_C_MASK;
        OBM[i].x = BIRD_X;
        OBM[i].y = Q9_6_to_sint16(bird->y);
    }
}

void bird_flap(bird_t *bird) { bird->y_velocity = BIRD_FLAP_STRENGTH; }

void bird_move(bird_t *bird) {
    if (bird->y == sint16_to_Q9_6(SCREEN_END - BIRD_HEIGHT)) return;

    bird->y_velocity = Q9_6_clamp(
        SINT_TO_Q9_6(-15), bird->y_velocity + BIRD_GRAVITY, SINT_TO_Q9_6(15));
    bird->y += bird->y_velocity;

    if (Q9_6_to_sint16(bird->y) > SCREEN_END - BIRD_HEIGHT) {
        bird->y = sint16_to_Q9_6(SCREEN_END - BIRD_HEIGHT);
        OBM[0].pattern_config = bird_squish_pattern_pmfa;
    }
}

void bird_draw(bird_t *bird) {
    uint8_t i;
    for (i = 0; i < BIRD_OBMAS_SIZE; i++) {
        OBM[i].y = Q9_6_to_sint8(bird->y);
    }
}