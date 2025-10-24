#include "map.inc"
#include "serialize.h"
#include <graphx.h>
#include <shared.h>
#include <stdint.h>

const float SCALE_X = 320.0 / 360.0;
const float SCALE_Y = 240.0 / 180.0;
const unsigned short ORIGIN_X = 180.0 * SCALE_X;
const unsigned short ORIGIN_Y = 90.0 * SCALE_Y;

static inline uint8_t get_pixel_color(uint8_t b, int shift) {
    uint8_t pixel = (b >> shift) & 3;
    switch (pixel) {
    case 0:
        return 0b00000000;
    case 1:
        return 0b00100101;
    case 2:
        return 0b01001001;
    case 3:
        return 0b01001001;
    }
    return 0;
}

void render_map(void) {
    uint8_t *ptr = (uint8_t *)gfx_vbuffer;
    for (unsigned int i = 0; i < sizeof(world_map); i++) {
        uint8_t b = world_map[i];
        for (int shift = 6; shift >= 0; shift -= 2) {
            *ptr++ = get_pixel_color(b, shift);
        }
    }
}

void guess(unsigned short x, unsigned short y) {
    float longitude = (float)(x + 1) / SCALE_X - 180.0;
    float latitude = 90.0 - (float)(y + 1) / SCALE_Y;

    PacketData guess = {.guess = {.coordinates = {.latitude = latitude, .longitude = longitude}}};
    Packet packet = {.data = guess, .tag = PACKET_GUESS};
    serialize_packet(&packet);
}

unsigned char convert_color(Color color) {
    switch (color) {
    case RED:
        return 0b11100000;
        break;
    case BLUE:
        return 0b00000011;
        break;
    case GREEN:
        return 0b00011100;
        break;
    case MAGENTA:
        return 0b11100011;
        break;
    case CYAN:
        return 0b00011111;
        break;
    case YELLOW:
        return 0b11111100;
        break;
    default:
        return 0xff;
        break;
    }
}

void draw_cursor(unsigned short x, unsigned short y, Color color) {
    gfx_SetColor(convert_color(color));
    gfx_FillRectangle(x, y, 4, 4);
}

void clear_cursor(short px, short py) {
    for (int y = 0; y < 4; y++) {
        int map_y = py + y;
        if (map_y < 0 || map_y >= GFX_LCD_HEIGHT)
            continue;

        unsigned int idx = (map_y * (GFX_LCD_WIDTH / 4)) + (px / 4);
        if (idx >= sizeof(world_map))
            continue;

        uint8_t b = world_map[idx];
        for (int bit = 0; bit < 4; bit++) {
            int shift = 6 - 2 * bit;
            gfx_SetColor(get_pixel_color(b, shift));
            gfx_SetPixel(px + bit, map_y);
        }
    }
}