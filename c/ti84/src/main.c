#include <keypadc.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <stdio.h>

#include <ti/getcsc.h>
#include <ti/screen.h>

#include "deserialize.h"
#include "device.h"
#include "graphx.h"
#include "serialize.h"
#include "shared.h"
#include "map.inc"

void send_init_packet() {
    PacketData data = {.init = {.options = {.color = YELLOW, .user = "tal"}}};
    Packet packet = {.data = data, .tag = PACKET_INIT};
    serialize_packet(&packet);
}

void ready() {
    PacketData ready = {.waiting_status = {.ready = true}};
    Packet packet = {.data = ready, .tag = PACKET_WAITING_STATUS};
    serialize_packet(&packet);
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

int main(void) {
    os_ClrHome();
    os_SetCursorPos(0, 0);
    os_PutStrFull("geoterm ti84");

    gfx_Begin();

    gfx_ZeroScreen();
    uint8_t *dst = gfx_vbuffer;
    for (int i = 0; i < sizeof(world_map); i++) {
        uint8_t b = world_map[i];
        *dst++ = gfx_palette[(b >> 6) & 3];
        *dst++ = gfx_palette[(b >> 4) & 3];
        *dst++ = gfx_palette[(b >> 2) & 3];
        *dst++ = gfx_palette[b & 3];
    }

    while (!os_GetCSC());
    gfx_End();

    // os_SetCursorPos(1, 0);
    // const usb_standard_descriptors_t *desc = srl_GetCDCStandardDescriptors();
    // usb_error_t usb_error = usb_Init(usb_handler, NULL, desc, USB_DEFAULT_INIT_FLAGS);
    // if (usb_error) {
    //     usb_Cleanup();
    //     os_PutStrFull("usb init error\n");
    //     do
    //         kb_Scan();
    //     while (!kb_IsDown(kb_KeyClear));
    //     return 1;
    // }

    // os_SetCursorPos(2, 0);
    // os_PutStrFull("init serial! :)");
    // os_SetCursorPos(3, 0);
    // os_PutStrFull("press enter");

    // while (!os_GetCSC()) {
    //     usb_HandleEvents();
    // };

    // send_init_packet();
    // Packet packet;
    // if (!wait(&packet, PACKET_CONFIRMED))
    //     return 1;

    // os_ClrHome();
    // os_SetCursorPos(0, 0);
    // printf("lobby size %d\n", packet.data.confirmed.lobby.len);
    // printf("name %s\n", packet.data.confirmed.options.user);
    // printf("press enter to ready.\n");

    // while (!os_GetCSC()) {
    //     usb_HandleEvents();
    // };
    // os_ClrHome();

    // ready();
    // os_ClrHome();
    // os_SetCursorPos(0, 0);

    // os_PutStrFull("ready!");
    // if (!wait(&packet, PACKET_ROUND_LOADING))
    //     return 1;

    // gfx_Begin();
    // gfx_ZeroScreen();
    // if (!wait(&packet, PACKET_ROUND))
    //     return 1;

    // init_palette();
    // gfx_SwapDraw();
    // while (!os_GetCSC()) {
        // usb_HandleEvents();
    // }

    // usb_Cleanup();
    return 0;
}
