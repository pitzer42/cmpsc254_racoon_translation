
#include "patterns.h"

void load_background_pattern(const pattern_t pattern, const uint8_t pmba) {
    uint8_t i;
    for (i = 0; i < 16; i++) PMB[pmba][i] = pattern[i];
}
void load_foreground_pattern(const pattern_t pattern, const uint8_t pmfa) {
    uint8_t i;
    for (i = 0; i < 16; i++) PMF[pmfa][i] = pattern[i];
}

void load_patterns(void) {
    load_foreground_pattern(white_pattern, white_pattern_pmfa);
    load_foreground_pattern(white_small_pattern, white_small_pattern_pmfa);
    load_foreground_pattern(bird_pattern, bird_pattern_pmfa);
    load_foreground_pattern(bird_dead_pattern, bird_dead_pattern_pmfa);
    load_foreground_pattern(bird_squish_pattern, bird_squish_pattern_pmfa);
    load_foreground_pattern(pipe_top_pattern, pipe_top_pattern_pmfa);
    load_foreground_pattern(pipe_body_pattern, pipe_body_pattern_pmfa);

    load_background_pattern(white_pattern, white_pattern_pmba);
    load_background_pattern(black_pattern, black_pattern_pmba);

    load_background_pattern(ground0_pattern, ground0_pattern_pmba);
    load_background_pattern(ground1_pattern, ground1_pattern_pmba);
    load_background_pattern(ground2_pattern, ground2_pattern_pmba);
    load_background_pattern(ground3_pattern, ground3_pattern_pmba);
    load_background_pattern(ground4_pattern, ground4_pattern_pmba);
}
