#pragma once
#include "shared.h"
#include <stdint.h>

extern const float SCALE_X;
extern const float SCALE_Y;
extern const unsigned short ORIGIN_X;
extern const unsigned short ORIGIN_Y;

void render_map(void);
void guess(unsigned short x, unsigned short y);
void draw_cursor(unsigned short x, unsigned short y, Color color);
void clear_cursor(short px, short py);