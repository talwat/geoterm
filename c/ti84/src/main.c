#include <graphx.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <string.h>

#include <ti/getcsc.h>
#include <ti/screen.h>

#include "deserialize.h"
#include "device.h"
#include "lobby.h"
#include "map.h"
#include "round.h"
#include "serialize.h"
#include "shared.h"
#include "utils.h"

const char *CSC_CHARS =
    "\0\0\0\0\0\0\0\0\0\0\"WRMH\0\0?[VQLG\0\0:ZUPKFC\0 YTOJEB\0\0XSNIDA\0\0\0\0\0\0\0\0";

bool prompt_name(char *buffer) {
    uint8_t key, i = 0;
    while ((key = os_GetCSC()) != sk_Enter) {
        if (key == sk_Clear)
            return false;

        usb_HandleEvents();

        if (CSC_CHARS[key]) {
            char print[2] = {CSC_CHARS[key], 0};
            os_PutStrLine(&print);
            buffer[i++] = CSC_CHARS[key];
        }
    }

    return true;
}

bool get_options(ClientOptions *options) {
    os_PutStrFull("username: ");
    memset(options->user, 0, 16);
    if (!prompt_name(options->user))
        return false;

    os_SetCursorPos(3, 0);
    os_PutStrFull("color:");
    os_SetCursorPos(4, 2);
    os_PutStrFull("0 = red      1 = green");
    os_SetCursorPos(5, 2);
    os_PutStrFull("2 = green    3 = cyan");
    os_SetCursorPos(6, 2);
    os_PutStrFull("4 = magenta  5 = yellow");

    uint8_t key = 0;
    while (!(key = os_GetCSC())) {
        usb_HandleEvents();
    }

    switch (key) {
    case sk_0:
        options->color = RED;
        break;
    case sk_1:
        options->color = BLUE;
        break;
    case sk_2:
        options->color = GREEN;
        break;
    }

    return true;
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

    ClientOptions options;
    if (!get_options(&options))
        return false;

    os_ClrHome();
    os_SetCursorPos(0, 0);
    os_PutStrFull("initialized serial! :)");

    os_SetCursorPos(2, 0);
    os_PutStrFull("controls: enter to guess, + to submit, clear to quit");

    os_SetCursorPos(5, 0);
    os_PutStrFull("press any key to continue");

    while (os_GetCSC())
        usb_HandleEvents();

    uint8_t key = 0;
    while (!key) {
        key = os_GetCSC();
        if (key == sk_Clear)
            return false;
        usb_HandleEvents();
    };

    PacketData data = {.init = {.options = options}};
    Packet packet = {.data = data, .tag = PACKET_INIT};
    serialize_packet(&packet);

    return true;
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
    init_palette();

    State state = STATE_LOBBY;

    while (true) {
        switch (state) {
        case STATE_LOBBY:
            if (!lobby(&packet)) {
                cleanup();
                return 0;
            }
            state = STATE_ROUND;
            break;
        case STATE_ROUND:
            if (!do_round(&state)) {
                cleanup();
                return 0;
            }
            break;
        }
    }

    cleanup();
    return 0;
}
