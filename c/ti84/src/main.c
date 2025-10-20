#include <keypadc.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <stdio.h>

#include <ti/getcsc.h>
#include <ti/screen.h>

#include "deserialize.h"
#include "device.h"
#include "graphx.h"
#include "map.inc"
#include "serialize.h"
#include "shared.h"

static bool READY = false;

int init() {
    os_ClrHome();
    os_SetCursorPos(0, 0);
    os_PutStrFull("geoterm ti84");

    os_SetCursorPos(1, 0);
    const usb_standard_descriptors_t *desc = srl_GetCDCStandardDescriptors();
    usb_error_t usb_error = usb_Init(usb_handler, NULL, desc, USB_DEFAULT_INIT_FLAGS);
    if (usb_error) {
        usb_Cleanup();
        os_PutStrFull("usb init error\n");
        do
            kb_Scan();
        while (!kb_IsDown(kb_KeyClear));
        return 1;
    }

    os_SetCursorPos(2, 0);
    os_PutStrFull("init serial! :)");

    os_SetCursorPos(3, 0);
    os_PutStrFull("press enter");

    while (!os_GetCSC()) {
        usb_HandleEvents();
    };

    PacketData data = {.init = {.options = {.color = GREEN, .user = "tal"}}};
    Packet packet = {.data = data, .tag = PACKET_INIT};
    serialize_packet(&packet);

    return 0;
}

void ready() {
    PacketData ready = {.waiting_status = {.ready = READY}};
    Packet packet = {.data = ready, .tag = PACKET_WAITING_STATUS};
    serialize_packet(&packet);
}

void clear_cursor(short px, short py) {
    for (int y = 0; y < 4; y++) {
        int map_y = py + y;
        if (map_y < 0 || map_y >= GFX_LCD_HEIGHT)
            continue;

        int byte_index = (map_y * (GFX_LCD_WIDTH / 4)) + (px / 4);
        if (byte_index >= sizeof(world_map))
            continue;

        uint8_t b = world_map[byte_index];
        for (int bit = 0; bit < 4; bit++) {
            uint8_t color_index = (b >> (6 - 2 * bit)) & 3;
            gfx_SetColor(gfx_palette[color_index]);
            gfx_SetPixel(px + bit, map_y);
        }
    }
}

bool wait(Packet *packet, PacketTag target) {
    while (true) {
        kb_Scan();
        if (kb_IsDown(kb_KeyClear)) {
            usb_Cleanup();
            return false;
        }

        if (!has_srl_device)
            return false;

        if (!deserialize_packet(packet)) {
            usb_HandleEvents();
            continue;
        }

        if (packet->tag == target)
            return true;
    }
}

void init_palette(void) {
    int idx = 0;
    for (int r = 0; r < 8; r++) {
        for (int g = 0; g < 8; g++) {
            for (int b = 0; b < 4; b++) {
                uint8_t R = (r * 255) / 7;
                uint8_t G = (g * 255) / 7;
                uint8_t B = (b * 255) / 3;
                gfx_palette[idx++] = gfx_RGBTo1555(R, G, B);
            }
        }
    }
}

void render_map() {
    uint8_t *dst = gfx_vbuffer;
    for (unsigned int i = 0; i < sizeof(world_map); i++) {
        uint8_t b = world_map[i];
        *dst++ = gfx_palette[(b >> 6) & 3];
        *dst++ = gfx_palette[(b >> 4) & 3];
        *dst++ = gfx_palette[(b >> 2) & 3];
        *dst++ = gfx_palette[b & 3];
    }
}

bool lobby(Packet *packet) {
    bool enter_prev = false;

    while (true) {
        while (true) {
            kb_Scan();
            if (kb_IsDown(kb_KeyEnter)) {
                if (!enter_prev) {
                    READY = !READY;
                    ready();
                }

                enter_prev = true;
            } else {
                enter_prev = false;
            }

            if (kb_IsDown(kb_KeyClear)) {
                usb_Cleanup();
                return false;
            }

            if (!has_srl_device)
                return false;

            if (!deserialize_packet(packet)) {
                usb_HandleEvents();
                continue;
            }

            if (packet->tag == PACKET_LOBBY_EVENT)
                break;

            if (packet->tag == PACKET_ROUND_LOADING)
                return true;
        }

        os_ClrHome();
        os_SetCursorPos(0, 0);
        os_PutStrFull("lobby:");
        for (int i = 0; i < LOBBY_LEN; i++) {
            os_SetCursorPos(i + 1, 0);
            os_PutStrFull("* ");
            os_PutStrFull(LOBBY[i].options.user);

            if (LOBBY[i].ready) {
                os_PutStrFull(" (ready)");
            }
        }
    }
}

int main(void) {
    if (init() != 0)
        return 1;

    Packet packet;
    if (!wait(&packet, PACKET_CONFIRMED))
        return 1;

    bool result = lobby(&packet);
    if (!result) {
        return 1;
    }

    gfx_Begin();
    init_palette();
    gfx_SetDrawScreen();
    gfx_SetColor(0xff);
    gfx_PrintStringXY("loading...", 8, 8);

    gfx_SetDrawBuffer();
    render_map();

    gfx_SetDrawScreen();
    if (!wait(&packet, PACKET_ROUND))
        return 1;
    gfx_SwapDraw();

    while (os_GetCSC()) {
        usb_HandleEvents();
    }

    const float SCALE_X = 320.0 / 360.0;
    const float SCALE_Y = 240.0 / 180.0;

    const short ORIGIN_X = 180.0 * SCALE_X;
    const short ORIGIN_Y = 90.0 * SCALE_Y;

    const short CURSOR_SPEED = 4;
    short cursor_x = ORIGIN_X, cursor_y = ORIGIN_Y;
    bool guesser = false;

    uint8_t key = 0;
    while ((key = os_GetCSC()) != sk_Clear) {
        usb_HandleEvents();
        if (key == sk_Enter) {
            gfx_SwapDraw();
            gfx_Wait();
            guesser = !guesser;
        }

        short prev_x = cursor_x;
        short prev_y = cursor_y;
        if (guesser && key != 0) {
            switch (key) {
            case sk_Left:
                cursor_x -= CURSOR_SPEED;
                break;
            case sk_Right:
                cursor_x += CURSOR_SPEED;
                break;
            case sk_Down:
                cursor_y += CURSOR_SPEED;
                break;
            case sk_Up:
                cursor_y -= CURSOR_SPEED;
                break;
            }

            gfx_SetDrawScreen();
            clear_cursor(prev_x, prev_y);
            gfx_SetColor(0b11100000);
            gfx_FillRectangle(cursor_x, cursor_y, 4, 4);
            gfx_SetDrawBuffer();
        }
    }

    gfx_End();
    usb_Cleanup();
    return 0;
}
