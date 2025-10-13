#include "decompress.h"
#include <stddef.h>
#include <stdint.h>

void lzss_decompress(const uint8_t *input, size_t input_len, uint8_t *output, size_t *out_len) {
    uint8_t window[N];
    size_t r = N - MAX_MATCH;
    for (size_t i = 0; i < r; i++)
        window[i] = C;

    size_t in_pos = 0;
    size_t out_pos = 0;

    while (in_pos < input_len) {
        uint8_t flags = input[in_pos++];
        for (int i = 0; i < 8 && in_pos < input_len; i++) {
            if (flags & 1) {
                uint8_t c = input[in_pos++];
                output[out_pos++] = c;
                window[r] = c;
                r = (r + 1) % N;
            } else {
                if (in_pos + 1 >= input_len)
                    break;
                uint8_t byte1 = input[in_pos++];
                uint8_t byte2 = input[in_pos++];

                uint16_t offset = ((byte1 << 4) | (byte2 >> 4)) & 0x03FF;
                uint16_t length = (byte2 & 0x0F) + THRESHOLD;

                for (int j = 0; j < length; j++) {
                    uint8_t c = window[(offset + j) % N];
                    output[out_pos++] = c;
                    window[r] = c;
                    r = (r + 1) % N;
                }
            }
            flags >>= 1;
        }
    }

    *out_len = out_pos;
}