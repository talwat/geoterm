#include "device.h"
#include <srldrvce.h>
#include <stdbool.h>
#include <tice.h>
#include <usbdrvce.h>

bool has_srl_device = false;
srl_device_t srl;
static uint8_t srl_buf[512];

usb_error_t usb_handler(
    usb_event_t event, void *event_data, usb_callback_data_t *callback_data __attribute__((unused))
) {
    usb_error_t err;
    if ((err = srl_UsbEventCallback(event, event_data, callback_data)) != USB_SUCCESS)
        return err;

    if (event == USB_DEVICE_CONNECTED_EVENT && !(usb_GetRole() & USB_ROLE_DEVICE)) {
        usb_device_t device = event_data;
        os_PutStrFull("device connected\n");
        usb_ResetDevice(device);
    }

    if (event == USB_HOST_CONFIGURE_EVENT ||
        (event == USB_DEVICE_ENABLED_EVENT && !(usb_GetRole() & USB_ROLE_DEVICE))) {
        if (has_srl_device)
            return USB_SUCCESS;

        usb_device_t device;
        if (event == USB_HOST_CONFIGURE_EVENT) {
            device = usb_FindDevice(NULL, NULL, USB_SKIP_HUBS);
            if (device == NULL)
                return USB_SUCCESS;
        } else {
            device = event_data;
        }

        srl_error_t error =
            srl_Open(&srl, device, srl_buf, sizeof srl_buf, SRL_INTERFACE_ANY, 9600);
        if (error) {
            os_PutStrFull("error initializing serial\n");
            return USB_SUCCESS;
        }

        has_srl_device = true;
    }

    if (event == USB_DEVICE_DISCONNECTED_EVENT) {
        usb_device_t device = event_data;
        if (device == srl.dev) {
            os_PutStrFull("device disconnected\n");
            srl_Close(&srl);
            has_srl_device = false;
        }
    }

    return USB_SUCCESS;
}
