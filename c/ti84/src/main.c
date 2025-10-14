#include <keypadc.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <stdio.h>

#include <ti/getcsc.h>
#include <ti/screen.h>

#include "deserialize.h"
#include "graphx.h"
#include "decompress.h"
#include "device.h"
#include "serialize.h"
#include "shared.h"

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

int main(void) {
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

    while (!os_GetCSC()) {
        usb_HandleEvents();
    };

    send_init_packet();
    os_PutStrFull("sent init packet");
    while (!os_GetCSC()) {
        usb_HandleEvents();
    };

    Packet packet;
    if (has_srl_device) {
        deserialize_packet(&packet);
    }

    os_SetCursorPos(3, 0);
    printf("lobby size %d\n", packet.data.confirmed.lobby.len);
    printf("name %s\n", packet.data.confirmed.options.user);

    while (!os_GetCSC()) {
        usb_HandleEvents();
    };
    os_ClrHome();

    ready();
    os_ClrHome();
    os_SetCursorPos(0, 0);
    os_PutStrFull("ready!");

    while (!os_GetCSC()) {
        usb_HandleEvents();
    };

    os_SetCursorPos(1, 0);
    do {
        if (has_srl_device) {
            deserialize_packet(&packet);
        } else {
            break;
        }

        switch (packet.tag) {
        case PACKET_LOBBY_EVENT:
            printf("lobby event\n");
            break;
        case PACKET_ROUND_LOADING:
            printf("loading...\n");
            break;
        case PACKET_ROUND:
            printf("round!\n");
            break;
        }

        while (!os_GetCSC()) {
            usb_HandleEvents();
        }
    } while (packet.tag != PACKET_ROUND);

    os_ClrHome();
    gfx_Begin();
    gfx_ZeroScreen();
    lzss_decompress_draw(packet.data.round.image, packet.data.round.image_len);
    
    while (!os_GetCSC()) {
        usb_HandleEvents();
    }

    gfx_End();
    usb_Cleanup();
    return 0;
}
