#include "pipearray.h"

#include <vram.h>

#include "bird.h"
#include "patterns.h"
#include "pipe.h"

pipe_t pipearray[PIPEARRAY_SIZE];

void pipearray_init(void) {
    uint8_t i, j;
    uint8_t k = BIRD_OBMAS_SIZE;

    for (i = 0; i < PIPEARRAY_SIZE; i++) {
        pipearray[i].holex = i * 64 + SCREEN_WIDTH - 16;
        pipe_newy(pipearray + i);

        pipearray[i].obmas[0] = k;
        OBM[k].pattern_config = pipe_top_pattern_pmfa;
        OBM[k].color = GREEN_C_MASK;
        k++;

        pipearray[i].obmas[1] = k;
        OBM[k].pattern_config = pipe_top_pattern_pmfa | VFLIP_MASK;
        OBM[k].color = GREEN_C_MASK;
        k++;

        for (j = 2; j < PIPE_OBMAS_SIZE; j++) {
            pipearray[i].obmas[j] = k;
            OBM[k].pattern_config = pipe_body_pattern_pmfa;
            OBM[k].color = GREEN_C_MASK;
            k++;
        }
    }
}

// returns true if 1 should be added to the score
bool pipearray_move(void) {
    uint8_t i;
    bool addscore = false;
    for (i = 0; i < PIPEARRAY_SIZE; i++)
        if (pipe_move(pipearray + i)) addscore = true;

    return addscore;
}

void pipearray_draw(void) {
    uint8_t i;
    for (i = 0; i < PIPEARRAY_SIZE; i++) pipe_draw(pipearray + i);
}

bool pipearray_collision(bird_t* bird) {
    uint8_t i;
    for (i = 0; i < PIPEARRAY_SIZE; i++)
        if (pipe_collision(pipearray + i, bird)) return true;

    return false;
}