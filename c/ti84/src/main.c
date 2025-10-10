#include <debug.h>
#include <graphx.h>
#include <keypadc.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <tice.h>

#include "device.h"
#include "shared.h"

char *read(char buf[128]) {
    size_t n = srl_Read(&srl, buf, 128);

    if (n <= 0) {
        return NULL;
    } else {
        return buf;
    }
}

void wait() {
    while (!kb_IsDown(kb_KeyClear)) {
        kb_Scan();
        usb_HandleEvents();
    }
}

int main(void) {
    os_ClrHome();
    os_SetCursorPos(0, 0);
    os_PutStrFull("geoterm ti84\ninitializing...\n");

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

    os_PutStrFull("initialized serial connection! :)");
    wait();

    PacketData data = {.init = {.options = {.color = 1, .user = "tal"}}};
    Packet packet = {.data = data, .type = PACKET_INIT};

    int n = srl_Write(&srl, &packet, sizeof(packet));
    printf("wrote %d bytes!", n);

    do {
        kb_Scan();
        usb_HandleEvents();

        if (has_srl_device) {
            // you should read here.
        }
    } while (!kb_IsDown(kb_KeyClear));

    usb_Cleanup();
    return 0;
}
