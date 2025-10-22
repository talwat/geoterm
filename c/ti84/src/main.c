#include <graphx.h>
#include <srldrvce.h>
#include <stdbool.h>

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
        usb_HandleEvents();

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
