#include <shared.h>
#include <stdbool.h>
#include <ti/getcsc.h>

#include "deserialize.h"
#include "device.h"
#include "graphx.h"
#include "utils.h"

inline void cleanup() {
    gfx_End();
    usb_Cleanup();
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