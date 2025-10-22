#include "shared.h"
#include <device.h>
#include <graphx.h>
#include <stdio.h>
#include <string.h>

LobbyClient LOBBY[16];
uint8_t LOBBY_LEN;
Player PLAYERS[16];

static uint8_t read_u8(void) {
    uint8_t v;
    srl_Read(&srl, &v, 1);
    return v;
}

static uint32_t read_u32(void) {
    uint8_t buf[4];
    srl_Read(&srl, buf, 4);
    return ((uint32_t)buf[0] << 24) | ((uint32_t)buf[1] << 16) | ((uint32_t)buf[2] << 8) |
           ((uint32_t)buf[3]);
}

static float read_f32(void) {
    uint8_t buf[4];
    srl_Read(&srl, buf, 4);
    uint32_t tmp = ((uint32_t)buf[0] << 24) | ((uint32_t)buf[1] << 16) | ((uint32_t)buf[2] << 8) |
                   ((uint32_t)buf[3]);
    float f;
    memcpy(&f, &tmp, 4);
    return f;
}

int read_all(void *buf, size_t len) {
    uint8_t *ptr = buf;
    size_t total = 0;

    while (total < len) {
        int n = srl_Read(&srl, ptr + total, len - total);
        if (n > 0) {
            total += n;
        } else {
            usb_HandleEvents();
        }
    }

    return total;
}

static void deserialize_client_options(ClientOptions *opt) {
    opt->color = (Color)read_u8();
    srl_Read(&srl, opt->user, 16);
}

static void deserialize_coordinate(Coordinate *c) {
    c->latitude = read_f32();
    c->longitude = read_f32();
}

static void deserialize_player(Player *p) {
    p->id = read_u32();
    p->points = read_u32();
    p->delta = read_u32();
    p->has_guess = read_u8();
    if (p->has_guess)
        deserialize_coordinate(&p->guess);
    else {
        uint8_t pad[8];
        srl_Read(&srl, pad, 8);
    }
}

static void deserialize_clients(Clients *lobby) {
    lobby->len = read_u32();
    LOBBY_LEN = lobby->len;
    lobby->clients = LOBBY;
    for (size_t i = 0; i < lobby->len; i++) {
        LobbyClient *c = &lobby->clients[i];
        c->id = read_u32();
        c->ready = read_u8();
        deserialize_client_options(&c->options);
    }
}

static void deserialize_result(RoundData *r) {
    r->number = read_u32();
    deserialize_coordinate(&r->answer);
    r->players_len = read_u32();
    r->players = PLAYERS;
    for (size_t i = 0; i < r->players_len; i++)
        deserialize_player(&r->players[i]);
}

bool deserialize_packet(Packet *p) {
    uint8_t tag;
    int n = srl_Read(&srl, &tag, 1);
    if (n <= 0) {
        return false;
    }

    p->tag = tag;
    // Stall for a bit to wait for all the data to come in.
    // TODO: Fix this shitty hack.
    for (volatile int i = 0; i < 50000; i++)
        usb_HandleEvents();

    switch (p->tag) {
    case PACKET_CONFIRMED:
        p->data.confirmed.id = read_u32();
        deserialize_client_options(&p->data.confirmed.options);
        deserialize_clients(&p->data.confirmed.lobby);
        break;
    case PACKET_LOBBY_EVENT:
        p->data.lobby_event.action = (LobbyAction)read_u8();
        p->data.lobby_event.user = read_u32();
        deserialize_clients(&p->data.lobby_event.lobby);
        break;
    case PACKET_ROUND_LOADING:
        deserialize_clients(&p->data.round_loading.lobby);
        break;
    case PACKET_ROUND:
        p->data.round.number = read_u32();
        p->data.round.image_len = read_u32();
        p->data.round.image = (unsigned char *)gfx_vbuffer;
        read_all(p->data.round.image, p->data.round.image_len - 1);
        break;
    case PACKET_GUESSED:
        p->data.guessed.player = read_u32();
        break;
    case PACKET_RESULT:
        deserialize_result(&p->data.results.round);
        break;
    default:
        break;
    }

    return true;
}