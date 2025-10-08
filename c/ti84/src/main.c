#include <debug.h>
#include <graphx.h>
#include <keypadc.h>
#include <srldrvce.h>
#include <stdbool.h>
#include <stdio.h>
#include <string.h>
#include <tice.h>

#include "device.h"

uint32_t assemble_u32_be(const char bytes[4]) {
    return ((uint32_t)bytes[0] << 24) | ((uint32_t)bytes[1] << 16) | ((uint32_t)bytes[2] << 8) |
           ((uint32_t)bytes[3]);
}

// Do NOT use this to read a packet that is very big.
char *read() {
    char len_bytes[4];
    srl_Read(&srl, &len_bytes, 4);
    uint32_t len = assemble_u32_be(len_bytes);

    char *buf = malloc(len);
    size_t n = srl_Read(&srl, buf, len);

    if (n <= 0) {
        return NULL;
    } else {
        return buf;
    }
}

int main(void) {
    os_ClrHome();
    os_SetCursorPos(0, 0);
    os_PutStrFull("geoterm ti84: initializing...");

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
    do {
        kb_Scan();
        usb_HandleEvents();

        if (has_srl_device) {
            char *buf = read();
        }
    } while (!kb_IsDown(kb_KeyClear));

    usb_Cleanup();
    return 0;
}
