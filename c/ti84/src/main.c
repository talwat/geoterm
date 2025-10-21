#include <graphx.h>
#include <srldrvce.h>
#include <stdbool.h>

#include <ti/getcsc.h>
#include <ti/screen.h>

#include "deserialize.h"
#include "device.h"
#include "lobby.h"
#include "map.h"
#include "serialize.h"
#include "shared.h"

void cleanup() {
    gfx_End();
    usb_Cleanup();
}

bool init() {
    os_ClrHome();
    os_SetCursorPos(0, 0);
    os_PutStrFull("geoterm ti84");

    os_SetCursorPos(1, 0);
    const usb_standard_descriptors_t *desc = srl_GetCDCStandardDescriptors();
    usb_error_t usb_error = usb_Init(usb_handler, NULL, desc, USB_DEFAULT_INIT_FLAGS);
    if (usb_error) {
        usb_Cleanup();
        os_PutStrFull("usb init error\n");
        while (os_GetCSC() != sk_Clear)
            return false;
    }

    // Stall to wait for transponder.
    for (volatile unsigned int i = 0; i < 100000; i++)
        usb_HandleEvents();

    os_SetCursorPos(1, 0);
    os_PutStrFull("initialized serial! :)");

    os_SetCursorPos(3, 0);
    os_PutStrFull("controls: enter to guess, + to submit, clear to quit");

    os_SetCursorPos(6, 0);
    os_PutStrFull("press any key to continue");

    while (os_GetCSC())
        ;
    uint8_t key = 0;
    while (!key) {
        key = os_GetCSC();
        if (key == sk_Clear)
            return false;
        usb_HandleEvents();
    };

    PacketData data = {.init = {.options = {.color = GREEN, .user = "tal"}}};
    Packet packet = {.data = data, .tag = PACKET_INIT};
    serialize_packet(&packet);

    return true;
}

bool wait(Packet *packet, PacketTag target) {
    while (has_srl_device) {
        if (os_GetCSC() == sk_Clear) {
            cleanup();
            return false;
        }

        if (!deserialize_packet(packet)) {
            usb_HandleEvents();
            continue;
        }

        if (packet->tag == target)
            return true;
    }

    return false;
}

void init_palette(void) {
    unsigned short idx = 0;
    for (uint8_t r = 0; r < 8; r++) {
        for (uint8_t g = 0; g < 8; g++) {
            for (int b = 0; b < 4; b++) {
                uint8_t R = (r * 255) / 7;
                uint8_t G = (g * 255) / 7;
                uint8_t B = (b * 255) / 3;
                gfx_palette[idx++] = gfx_RGBTo1555(R, G, B);
            }
        }
    }
}

int main(void) {
    if (!init()) {
        cleanup();
        return 1;
    }

    Packet packet;
    if (!wait(&packet, PACKET_CONFIRMED))
        return 1;

    gfx_Begin();
    bool result = lobby(&packet);
    if (!result) {
        cleanup();
        return 1;
    }

    init_palette();

    gfx_SetDrawScreen();
    gfx_FillScreen(0xff);
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

    const unsigned short CURSOR_SPEED = 4;
    unsigned short cursor_x = ORIGIN_X, cursor_y = ORIGIN_Y;
    bool guesser = false;

    while (true) {
        uint8_t key = os_GetCSC();
        if (key == sk_Clear) {
            cleanup();
            return 0;
        }

        usb_HandleEvents();
        if (key == sk_Add) {
            guess(cursor_x, cursor_y);
            break;
        }

        if (key == sk_Enter) {
            gfx_SwapDraw();
            gfx_Wait();
            guesser = !guesser;
        }

        if (guesser && key != 0) {
            gfx_SetDrawScreen();
            clear_cursor(cursor_x, cursor_y);
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

            draw_cursor(cursor_x, cursor_y, RED);
            gfx_SetDrawBuffer();
        }
    }

    gfx_SetDrawScreen();
    // gfx_SetDefaultPalette(gfx_8bpp);
    gfx_FillScreen(0xff);
    gfx_PrintStringXY("waiting for other guesses...", 8, 8);
    if (!wait(&packet, PACKET_RESULT))
        return 1;

    gfx_SetDrawBuffer();
    render_map();
    for (unsigned int i = 0; i < packet.data.result.round.players_len; i++) {
        Player player = packet.data.result.round.players[i];

        if (!player.has_guess) {
            continue;
        }

        unsigned short x = (player.guess.longitude + 180.0) * SCALE_X;
        unsigned short y = (90.0 - player.guess.latitude) * SCALE_Y;
        draw_cursor(x, y, LOBBY[i].options.color);
    }

    Coordinate answer = packet.data.result.round.answer;
    unsigned short x = (answer.longitude + 180.0) * SCALE_X;
    unsigned short y = (90.0 - answer.latitude) * SCALE_Y;
    draw_cursor(x, y, RED);

    gfx_SwapDraw();
    gfx_Wait();
    while (!os_GetCSC()) {
        usb_HandleEvents();
    }

    cleanup();
    return 0;
}
