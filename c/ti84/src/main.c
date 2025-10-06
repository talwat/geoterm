#include <srldrvce.h>
#include <debug.h>
#include <keypadc.h>
#include <stdbool.h>
#include <string.h>
#include <tice.h>
#include <graphx.h>
#include <stdio.h>

srl_device_t srl;
bool has_srl_device = false;
uint8_t srl_buf[512];

static usb_error_t handle_usb_event(usb_event_t event, void *event_data,
                                    usb_callback_data_t *callback_data __attribute__((unused))) {
    usb_error_t err;

    /* Delegate to srl USB callback */
    if ((err = srl_UsbEventCallback(event, event_data, callback_data)) != USB_SUCCESS)
        return err;

    if(event == USB_DEVICE_CONNECTED_EVENT && !(usb_GetRole() & USB_ROLE_DEVICE)) {
        usb_device_t device = event_data;
        usb_ResetDevice(device);
    }

    if(event == USB_HOST_CONFIGURE_EVENT || (event == USB_DEVICE_ENABLED_EVENT && !(usb_GetRole() & USB_ROLE_DEVICE))) {

        if(has_srl_device) return USB_SUCCESS;

        usb_device_t device;
        if(event == USB_HOST_CONFIGURE_EVENT) {
            device = usb_FindDevice(NULL, NULL, USB_SKIP_HUBS);
            if(device == NULL) return USB_SUCCESS;
        } else {
            device = event_data;
        }

        srl_error_t error = srl_Open(&srl, device, srl_buf, sizeof srl_buf, SRL_INTERFACE_ANY, 9600);
        if(error) {
            os_PutStrFull("Error initializing serial\n");
            return USB_SUCCESS;
        }

        os_PutStrFull("Serial initialized\n");
        has_srl_device = true;
    }

    if(event == USB_DEVICE_DISCONNECTED_EVENT) {
        usb_device_t device = event_data;
        if(device == srl.dev) {
            os_PutStrFull("Device disconnected\n");
            srl_Close(&srl);
            has_srl_device = false;
        }
    }

    return USB_SUCCESS;
}

int main(void) {
    os_ClrHome();
    const usb_standard_descriptors_t *desc = srl_GetCDCStandardDescriptors();
    usb_error_t usb_error = usb_Init(handle_usb_event, NULL, desc, USB_DEFAULT_INIT_FLAGS);
    if(usb_error) {
       usb_Cleanup();
       os_PutStrFull("USB init error\n");
       do kb_Scan(); while(!kb_IsDown(kb_KeyClear));
       return 1;
    }

    char screen_buf[256];
    int line = 0;

    do {
        kb_Scan();
        usb_HandleEvents();

        if(has_srl_device) {
            char in_buf[64];

            /* Read up to 64 bytes from the serial buffer */
            size_t bytes_read = srl_Read(&srl, in_buf, sizeof in_buf);

            if(bytes_read < 0) {
                os_PutStrFull("Error reading serial\n");
                has_srl_device = false;
            } else if(bytes_read > 0) {
                /* Echo back to serial */
                srl_Write(&srl, in_buf, bytes_read);

                /* Append incoming text to screen buffer */
                memcpy(screen_buf, in_buf, bytes_read);
                screen_buf[bytes_read] = 0;

                /* Move to next line if needed */
                if(line >= 8) {
                    os_ClrHome();
                    line = 0;
                }

                os_SetCursorPos(line, 0);
                os_PutStrFull(screen_buf);
                line++;
            }
        }

    } while(!kb_IsDown(kb_KeyClear));

    usb_Cleanup();
    return 0;
}
