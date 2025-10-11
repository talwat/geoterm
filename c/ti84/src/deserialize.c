#include "shared.h"
#include <device.h>
#include <string.h>

static const char IMAGE[IMAGE_SIZE];
static const LobbyClient LOBBY[16];
static const Player PLAYERS[16];

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
    p->has_guess = read_u8();
    if (p->has_guess)
        deserialize_coordinate(&p->guess);
    else {
        uint8_t pad[2];
        srl_Read(&srl, pad, 2);
    }
}

static void deserialize_clients(Clients *lobby) {
    lobby->len = read_u32();
    lobby->clients = LOBBY;
    for (size_t i = 0; i < lobby->len; i++) {
        LobbyClient *c = &lobby->clients[i];
        c->id = read_u32();
        c->ready = read_u8();
        deserialize_client_options(&c->options);
    }
}

static void deserialize_round_data(RoundData *r) {
    r->number = read_u32();
    deserialize_coordinate(&r->answer);
    r->players_len = read_u32();
    r->players = PLAYERS;
    for (size_t i = 0; i < r->players_len; i++)
        deserialize_player(&r->players[i]);
}

void deserialize_packet(Packet *p) {
    p->id = (PacketType)read_u8();
    switch (p->id) {
    case PACKET_INIT:
        deserialize_client_options(&p->data.init.options);
        break;
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
    case PACKET_WAITING_STATUS:
        p->data.waiting_status.ready = read_u8();
        break;
    case PACKET_ROUND_LOADING:
        deserialize_clients(&p->data.round_loading.lobby);
        break;
    case PACKET_ROUND:
        p->data.round.number = read_u32();
        p->data.round.image_len = read_u32();
        p->data.round.image = IMAGE;
        srl_Read(&srl, p->data.round.image, p->data.round.image_len);
        break;
    case PACKET_GUESS:
        deserialize_coordinate(&p->data.guess.coordinates);
        break;
    case PACKET_GUESSED:
        p->data.guessed.player = read_u32();
        break;
    case PACKET_RESULT:
        deserialize_round_data(&p->data.result.round);
        break;
    default:
        break;
    }
}