#include "shared.h"
#include <device.h>
#include <stdint.h>
#include <string.h>

static void write_u8(uint8_t v) { srl_Write(&srl, &v, 1); }

static void write_f32(float f) {
    uint32_t tmp;
    memcpy(&tmp, &f, 4);
    uint8_t buf[4] = {(tmp >> 24) & 0xFF, (tmp >> 16) & 0xFF, (tmp >> 8) & 0xFF, tmp & 0xFF};
    srl_Write(&srl, buf, 4);
}

static void serialize_client_options(const ClientOptions *opt) {
    write_u8((uint8_t)opt->color);
    srl_Write(&srl, opt->user, 16);
}

static void serialize_coordinate(const Coordinate *c) {
    write_f32(c->latitude);
    write_f32(c->longitude);
}

void serialize_packet(const Packet *p) {
    write_u8((uint8_t)p->tag);
    switch (p->tag) {
    case PACKET_INIT:
        serialize_client_options(&p->data.init.options);
        break;
    case PACKET_WAITING_STATUS:
        write_u8(p->data.waiting_status.ready);
        break;
    case PACKET_GUESS:
        serialize_coordinate(&p->data.guess.coordinates);
        break;
    default:
        break;
    }
}