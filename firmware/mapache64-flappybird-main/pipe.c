#include "pipe.h"

#include <mapache64_zero_page.h>
#include <stdlib.h>
#include <stop.h>
#include <vram.h>

#include "bird.h"
#include "patterns.h"

// returns true if 1 should be added to the score
bool pipe_move(pipe_t* p) {
    p->holex -= 1;
    if (p->holex < 0) {
        p->holex = SCREEN_WIDTH - 1;
        pipe_newy(p);
        p->passed = false;
    } else if (p->holex < BIRD_X && !(p->passed)) {
        p->passed = true;
        return true;
    }

    return false;
}

bool pipe_collision(pipe_t* p, bird_t* bird) {
    if (BIRD_X < p->holex - BIRD_WIDTH || BIRD_X > p->holex + PIPE_WIDTH)
        return false;

    return p->holey > Q9_6_to_sint16(bird->y) ||
           Q9_6_to_sint16(bird->y) + BIRD_HEIGHT > p->holey + PIPE_HOLE_HEIGHT;
}

void pipe_draw(pipe_t* p) {
    uint8_t j;

    if (p->holex >= SCREEN_WIDTH) {
        for (j = 0; j < PIPE_OBMAS_SIZE; j++) {
            OBM[p->obmas[j]].x = 0;
            OBM[p->obmas[j]].y = 255;
        }
    }

    else {
        for (j = 0; j < PIPE_OBMAS_SIZE; j++) {
            OBM[p->obmas[j]].x = p->holex;
        }

        if (OBM[p->obmas[0]].y != p->holey - 8 &&
            OBM[p->obmas[0]].y !=
                p->holey - 8 + SCREEN_HEIGHT) {  // if y value has changed
            uint8_t newy = p->holey;
            OBM[p->obmas[0]].y = p->holey - 8;
            OBM[p->obmas[1]].y = p->holey + PIPE_HOLE_HEIGHT;
            for (j = 2; j < PIPE_OBMAS_SIZE; j++) {
                if (newy != p->holey + PIPE_HOLE_HEIGHT && newy != 255)
                    newy -= 8;

                if (newy < SCREEN_START) {
                    newy = SCREEN_START;
                    OBM[p->obmas[j]].y = newy;
                    newy = SCREEN_END;
                } else if (newy > p->holey &&
                           newy < p->holey + PIPE_HOLE_HEIGHT) {
                    newy = 255;
                    OBM[p->obmas[j]].y = newy;
                } else
                    OBM[p->obmas[j]].y = newy;
            }
        }
    }
}

void pipe_newy(pipe_t* p) {
    p->holey = SCREEN_START + 8 + rand() % (SCREEN_HEIGHT - 40);
    p->passed = false;
}
