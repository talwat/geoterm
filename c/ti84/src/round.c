#include <graphx.h>
#include <shared.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <ti/getcsc.h>

#include "deserialize.h"
#include "device.h"
#include "lobby.h"
#include "map.h"
#include "serialize.h"
#include "utils.h"

void draw_coord(Coordinate coord, Color color, bool cross) {
    short x = (coord.longitude + 180.0) * SCALE_X;
    short y = (90.0 - coord.latitude) * SCALE_Y;

    if (cross) {
        gfx_SetColor(convert_color(color));
        gfx_Line(x + 4, y + 4, x - 4, y - 4);
        gfx_Line(x - 4, y + 4, x + 4, y - 4);
    } else {
        draw_cursor(x, y, color);
    }
}

bool do_round(State *state) {
    Packet packet;

    gfx_SetDrawScreen();
    gfx_FillScreen(0xff);
    gfx_PrintStringXY("loading...", 8, 8);

    gfx_SetDrawBuffer();
    render_map();

    gfx_SetDrawScreen();
    if (!wait(&packet, PACKET_ROUND))
        return false;
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
            return false;
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
    gfx_FillScreen(0xff);
    gfx_PrintStringXY("waiting for other guesses...", 8, 8);
    if (!wait(&packet, PACKET_RESULT))
        return false;

    gfx_SetDrawBuffer();
    render_map();
    for (unsigned int i = 0; i < packet.data.results.round.players_len; i++) {
        Player player = packet.data.results.round.players[i];

        if (!player.has_guess) {
            continue;
        }

        draw_coord(player.guess, LOBBY[i].options.color, false);
    }

    Coordinate answer = packet.data.results.round.answer;
    draw_coord(answer, RED, true);

    gfx_SwapDraw();
    gfx_Wait();
    while (!os_GetCSC()) {
        usb_HandleEvents();
    }

    gfx_SetDrawScreen();
    gfx_FillScreen(0xff);
    gfx_PrintStringXY("press enter to play again", 8, 8);
    gfx_PrintStringXY("press + to return to lobby", 8, 18);
    uint8_t key;
    while (!(key = os_GetCSC())) {
        usb_HandleEvents();
    }

    switch (key) {
    case sk_Enter:
        send_ready(true);
        gfx_FillScreen(0xff);
        gfx_PrintStringXY("waiting for others...", 8, 8);

        while (has_srl_device) {
            if (os_GetCSC() == sk_Clear) {
                return false;
            }

            if (!deserialize_packet(&packet)) {
                usb_HandleEvents();
                continue;
            }

            switch (packet.tag) {
            case PACKET_ROUND_LOADING:
                *state = STATE_ROUND;
                return true;
                break;
            case PACKET_LOBBY_EVENT:
                if (packet.data.lobby_event.action != LOBBY_RETURN)
                    break;

                *state = STATE_LOBBY;
                return true;
                break;
            default:
                break;
            }
        }

        return true;
        break;
    case sk_Add:
        packet = (Packet){.tag = PACKET_REQUEST_GAME_END, .data = {}};
        serialize_packet(&packet);
        *state = STATE_LOBBY;
        return true;
        break;
    default:
        return false;
        break;
    }
}