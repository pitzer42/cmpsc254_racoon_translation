
#ifndef __PATTERNS_H
#define __PATTERNS_H

#include <vram.h>

extern const pattern_t white_pattern;
extern const pattern_t white_small_pattern;
extern const pattern_t black_pattern;
extern const pattern_t bird_pattern;
extern const pattern_t bird_dead_pattern;
extern const pattern_t bird_squish_pattern;
extern const pattern_t pipe_top_pattern;
extern const pattern_t pipe_body_pattern;
extern const pattern_t ground0_pattern;
extern const pattern_t ground1_pattern;
extern const pattern_t ground2_pattern;
extern const pattern_t ground3_pattern;
extern const pattern_t ground4_pattern;

#define white_pattern_pmfa ((uint8_t)0)
#define white_small_pattern_pmfa ((uint8_t)1)
#define bird_pattern_pmfa ((uint8_t)2)
#define bird_dead_pattern_pmfa ((uint8_t)5)
#define bird_squish_pattern_pmfa ((uint8_t)6)
#define pipe_top_pattern_pmfa ((uint8_t)3)
#define pipe_body_pattern_pmfa ((uint8_t)4)

#define white_pattern_pmba ((uint8_t)0)
#define black_pattern_pmba ((uint8_t)1)
#define ground0_pattern_pmba ((uint8_t)2)
#define ground1_pattern_pmba ((uint8_t)3)
#define ground2_pattern_pmba ((uint8_t)4)
#define ground3_pattern_pmba ((uint8_t)5)
#define ground4_pattern_pmba ((uint8_t)6)

void load_background_pattern(const pattern_t pattern, const uint8_t pmba);
void load_foreground_pattern(const pattern_t pattern, const uint8_t pmfa);

void load_patterns(void);

#endif
