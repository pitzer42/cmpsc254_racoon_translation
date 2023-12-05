#ifndef __PIPEARRAY_H
#define __PIPEARRAY_H

#include "pipe.h"

#define PIPEARRAY_SIZE 4

extern pipe_t pipearray[PIPEARRAY_SIZE];

void pipearray_init(void);
void pipearray_draw(void);
bool pipearray_move(void);
bool pipearray_collision(bird_t* bird);

#endif