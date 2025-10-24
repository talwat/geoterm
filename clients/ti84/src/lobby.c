#include "deserialize.h"
#include "device.h"
#include "serialize.h"
#include <graphx.h>
#include <shared.h>
#include <stdio.h>
#include <ti/getcsc.h>

void send_ready(bool ready) {
    PacketData data = {.waiting_status = {.ready = ready}};
    Packet packet = {.data = data, .tag = PACKET_WAITING_STATUS};
    serialize_packet(&packet);
}

bool lobby(Packet *packet) {
    bool ready = false;
    gfx_SetDrawBuffer();

    while (true) {
        uint8_t key = os_GetCSC();
        if (key == sk_Clear || !has_srl_device) {
            return false;
        }

        if (key == sk_Enter) {
            ready = !ready;
            send_ready(ready);
        }

        if (deserialize_packet(packet) && packet->tag == PACKET_ROUND_LOADING)
            return true;
        else
            usb_HandleEvents();

        gfx_FillScreen(0xff);
        gfx_PrintStringXY("lobby:", 8, 8);
        for (int i = 0; i < LOBBY_LEN; i++) {
            int y = (i + 1) * 12 + (8);
            char *ready = "\0";
            if (LOBBY[i].ready) {
                ready = " (ready)";
            }

            char string[24];
            sprintf(string, "* %s%s", LOBBY[i].options.user, ready);
            gfx_PrintStringXY(string, 8, y);
        }
        gfx_SwapDraw();
    }

    return true;
}