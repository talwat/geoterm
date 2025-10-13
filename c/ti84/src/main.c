#include <keypadc.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <stdio.h>

#include <ti/getcsc.h>
#include <ti/screen.h>

#include "deserialize.h"
#include "device.h"
#include "serialize.h"
#include "shared.h"

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

    PacketData data = {.init = {.options = {.color = YELLOW, .user = "tal"}}};
    Packet packet = {.data = data, .tag = PACKET_INIT};
    serialize_packet(&packet);

    os_SetCursorPos(3, 0);
    printf("sent packet with type %d", packet.tag);
    os_SetCursorPos(4, 0);

    while (!os_GetCSC()) {
        usb_HandleEvents();
    };

    if (has_srl_device) {
        deserialize_packet(&packet);
        printf("got packet of tag %d\n", packet.tag);
        printf("lobby size %d\n", packet.data.confirmed.lobby.len);
        printf("name %s\n", packet.data.confirmed.options.user);
    }

    while (!os_GetCSC()) {
        usb_HandleEvents();
    };

    usb_Cleanup();
    return 0;
}
