#include "shared.h"
#include <device.h>
#include <string.h>

static void write_u8(uint8_t v) { srl_Write(&srl, &v, 1); }
static void write_u32(uint32_t v) {
    uint8_t buf[4] = {(v >> 24) & 0xFF, (v >> 16) & 0xFF, (v >> 8) & 0xFF, v & 0xFF};
    srl_Write(&srl, buf, 4);
}

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

static void serialize_player(const Player *p) {
    write_u32((uint32_t)p->id);
    write_u32((uint32_t)p->points);
    write_u8(p->has_guess);
    if (p->has_guess)
        serialize_coordinate(&p->guess);
    else {
        uint8_t pad[2] = {0};
        srl_Write(&srl, pad, 2);
    }
}

static void serialize_clients(const Clients *lobby) {
    write_u32((uint32_t)lobby->len);
    for (size_t i = 0; i < lobby->len; i++) {
        const LobbyClient *c = &lobby->clients[i];
        write_u32((uint32_t)c->id);
        write_u8(c->ready);
        serialize_client_options(&c->options);
    }
}

static void serialize_round_data(const RoundData *r) {
    write_u32((uint32_t)r->number);
    serialize_coordinate(&r->answer);
    write_u32((uint32_t)r->players_len);
    for (size_t i = 0; i < r->players_len; i++)
        serialize_player(&r->players[i]);
}

void serialize_packet(const Packet *p) {
    write_u8((uint8_t)p->id);
    switch (p->id) {
    case PACKET_INIT:
        serialize_client_options(&p->data.init.options);
        break;
    case PACKET_CONFIRMED:
        write_u32((uint32_t)p->data.confirmed.id);
        serialize_client_options(&p->data.confirmed.options);
        serialize_clients(&p->data.confirmed.lobby);
        break;
    case PACKET_LOBBY_EVENT:
        write_u8((uint8_t)p->data.lobby_event.action);
        write_u32((uint32_t)p->data.lobby_event.user);
        serialize_clients(&p->data.lobby_event.lobby);
        break;
    case PACKET_WAITING_STATUS:
        write_u8(p->data.waiting_status.ready);
        break;
    case PACKET_ROUND_LOADING:
        serialize_clients(&p->data.round_loading.lobby);
        break;
    case PACKET_ROUND:
        write_u32((uint32_t)p->data.round.number);
        write_u32((uint32_t)p->data.round.image_len);
        srl_Write(&srl, p->data.round.image, p->data.round.image_len);
        break;
    case PACKET_GUESS:
        serialize_coordinate(&p->data.guess.coordinates);
        break;
    case PACKET_GUESSED:
        write_u32((uint32_t)p->data.guessed.player);
        break;
    case PACKET_RESULT:
        serialize_round_data(&p->data.result.round);
        break;
    default:
        break;
    }
}