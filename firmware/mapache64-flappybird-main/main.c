
// ==================== main.c ==================== //
// Follow this file template to implement your game //
// ================================================ //

#define SIM 1  // change value depending if running simulation or not

// ================================ //
#pragma code - name("CODE")
#pragma bss - name("BSS")

#include <Q9_6.h>
#include <controller.h>
#include <int.h>
#include <mapache64_zero_page.h>
#include <screen.h>
#include <stop.h>
#include <vram.h>

#include "bird.h"
#include "patterns.h"
#include "pipe.h"
#include "pipearray.h"

bird_t bird;
bool game_running = false;
bool show_title;
bool bobup;
uint8_t bob_phase;
uint8_t score;
bool vram_initialized = false;
bool movepipe;
uint8_t ground_phase;
uint8_t hiscore;

const char title[] = "Flappy Bird";
const char gameover[] = "Game Over!";
const char hiscore_text[] = "High score: ";

uint16_t CONTROLLER_1_PEDGE;
uint16_t CONTROLLER_1_PREV;

void fill_vram(void);

void draw_score(void) {
    if (score < 10)
        TXBL[4][16] = score + '0' | COLOR_SELECT_MASK;
    else if (score < 100) {
        TXBL[4][16] = score % 10 + '0' | COLOR_SELECT_MASK;
        TXBL[4][15] = score / 10 + '0' | COLOR_SELECT_MASK;
    } else {
        TXBL[4][16] = score % 10 + '0' | COLOR_SELECT_MASK;
        TXBL[4][15] = (score / 10) % 10 + '0' | COLOR_SELECT_MASK;
        TXBL[4][14] = score % 100 + '0' | COLOR_SELECT_MASK;
    }
}

void draw_hiscore(void) {
    uint8_t i;

    if(hiscore > 0) {
        for(i = 0; i < sizeof(hiscore_text); i++)
            TXBL[26][i + 9] = hiscore_text[i] | COLOR_SELECT_MASK;

        if (hiscore < 10)
            TXBL[26][i + 9] = hiscore + '0' | COLOR_SELECT_MASK;
        else if (hiscore < 100) {
            TXBL[26][i + 10] = hiscore % 10 + '0' | COLOR_SELECT_MASK;
            TXBL[26][i + 9] = hiscore / 10 + '0' | COLOR_SELECT_MASK;
        } else {
            TXBL[26][i + 11] = hiscore % 10 + '0' | COLOR_SELECT_MASK;
            TXBL[26][i + 10] = (hiscore / 10) % 10 + '0' | COLOR_SELECT_MASK;
            TXBL[26][i + 9] = hiscore % 100 + '0' | COLOR_SELECT_MASK;
        }
    }
}

void reset_TXBL(void) {
    uint8_t i, j;
    for (i = 0; i < 30; i++)
        for (j = 0; j < 32; j++) TXBL[i][j] = 0;
}

void show_gameover(void) {
    uint8_t i;
    for (i = 0; i < sizeof(gameover) / sizeof(gameover[0]); i++)
        TXBL[2][i + 11] = gameover[i] | COLOR_SELECT_MASK;
}

void move_ground() {
    tile_t pmba;
    uint8_t j;

    switch (ground_phase) {
        case 0: pmba = ground0_pattern_pmba; break;
        case 1: pmba = ground1_pattern_pmba; break;
        case 2: pmba = ground2_pattern_pmba; break;
        case 3: pmba = ground3_pattern_pmba; break;
        case 4: pmba = ground4_pattern_pmba; break;
        case 5: pmba = ground3_pattern_pmba|HFLIP_MASK|VFLIP_MASK; break;
        case 6: pmba = ground2_pattern_pmba|HFLIP_MASK|VFLIP_MASK; break;
        case 7: pmba = ground1_pattern_pmba|HFLIP_MASK|VFLIP_MASK; break;
        default: break;
    }

    for (j = 0; j < 32; j++) NTBL[SCREEN_END / 8][j] = pmba;

    ground_phase = (ground_phase+1) & 0x7;

}

void init_vram(void) {
    uint8_t i, j;
    // clear vram
    for (i = 0; i < 64; i++) {
        OBM[i].y = 0xff;
    }
    for (i = 0; i < 31; i++)
        for (j = 0; j < 15; j++) {
            PMF[i][j] = 0;
            PMB[i][j] = 0;
        }
    reset_TXBL();

    load_patterns();

    // load background
    background_palette = GREEN_C0_MASK | CYAN_C1_MASK;
    for (i = 0; i < SCREEN_START / 8; i++)
        for (j = 0; j < 32; j++) NTBL[i][j] = black_pattern_pmba;

    for (i = SCREEN_START / 8; i < SCREEN_END / 8; i++)
        for (j = 0; j < 32; j++)
            NTBL[i][j] = white_pattern_pmba | COLOR_SELECT_MASK;

    ground_phase = 0;
    move_ground();

    for (i = SCREEN_END / 8 + 1; i < GameHeight / 8; i++)
        for (j = 0; j < 32; j++) NTBL[i][j] = black_pattern_pmba;

    show_title = true;
    // show title screen text
    for (i = 0; i < sizeof(title) / sizeof(title[0]); i++)
        TXBL[3][i + 10] = title[i] | COLOR_SELECT_MASK;
    bird_init(&bird);
    bird_draw(&bird);

    vram_initialized = true;
}

// run once on startup
void reset(void) {
    if (!vram_initialized) init_vram();
}

// run 60 times a second
void do_logic(void) {
    // calculate positive edge of all controller buttons
    CONTROLLER_1_PEDGE = (~CONTROLLER_1_PREV) & CONTROLLER_1;
    CONTROLLER_1_PREV = CONTROLLER_1;

    if (game_running) {
        if (movepipe && pipearray_move()) {
            score++;
            draw_score();
        }

        if ((CONTROLLER_1_PEDGE & CONTROLLER_A_MASK) &&
            Q9_6_to_sint16(bird.y) > SCREEN_START)
            bird_flap(&bird);

        bird_move(&bird);

        if (pipearray_collision(&bird) ||
            Q9_6_to_sint16(bird.y) >= SCREEN_END - BIRD_HEIGHT) {
            game_running = false;
            show_gameover();
            OBM[0].pattern_config = bird_dead_pattern_pmfa;
        }

        if (FRAME&1) move_ground();
        pipearray_draw();
        bird_draw(&bird);

        movepipe = !movepipe;

    } else if ((CONTROLLER_1_PEDGE & CONTROLLER_A_MASK) &&
               (Q9_6_to_sint16(bird.y) == SCREEN_END - BIRD_HEIGHT ||
                show_title)) {
        if(score > hiscore)
            hiscore = score;

        reset_TXBL();
        score = 0;
        draw_score();
        draw_hiscore();
        pipearray_init();
        bird_init(&bird);
        game_running = true;
        show_title = false;
    } else if (show_title) {
        if (FRAME&1) move_ground();

        if (bob_phase == 0) {
            bird.y = sint16_to_Q9_6(SCREEN_START + (SCREEN_HEIGHT / 2) +
                                    (bobup ? 3 : 0));
            bird_draw(&bird);
            bobup = !bobup;
        }

        bob_phase++;
        if (bob_phase > 20) bob_phase = 0;
    } else {
        bird_move(&bird);
        bird_draw(&bird);
    }
}

// run after do_logic and once gpu is idle
void fill_vram(void) {}
