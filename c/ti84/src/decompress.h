#ifndef LZSS_DECODER_H
#define LZSS_DECODER_H

#include <stddef.h>
#include <stdint.h>

#define EI 10
#define EJ 4
#define C 0x20
#define N (1 << EI)
#define MAX_MATCH ((1 << EJ) + 2)
#define THRESHOLD 2

void lzss_decompress_draw(const uint8_t *input, size_t input_len);

#endif
