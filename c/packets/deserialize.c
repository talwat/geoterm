#include "shared.h"
#include <arpa/inet.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

static uint8_t read_u8(int fd) {
    uint8_t v;
    read(fd, &v, 1);
    return v;
}
static uint32_t read_u32(int fd) {
    uint32_t v;
    read(fd, &v, 4);
    return ntohl(v);
}
static float read_f32(int fd) {
    float f;
    read(fd, &f, 4);
    return f;
}

static void deserialize_client_options(int fd, ClientOptions *opt) {
    opt->color = (Color)read_u8(fd);
    read(fd, opt->user, 16);
}

static void deserialize_coordinate(int fd, Coordinate *c) {
    c->latitude = read_f32(fd);
    c->longitude = read_f32(fd);
}

static void deserialize_player(int fd, Player *p) {
    p->id = read_u32(fd);
    p->points = read_u32(fd);
    p->has_guess = read_u8(fd);
    if (p->has_guess)
        deserialize_coordinate(fd, &p->guess);
    else {
        uint8_t pad[2];
        read(fd, pad, 2);
    }
}

static void deserialize_clients(int fd, Clients *lobby) {
    lobby->len = read_u32(fd);
    lobby->clients = malloc(lobby->len * sizeof(LobbyClient));
    for (size_t i = 0; i < lobby->len; i++) {
        LobbyClient *c = &lobby->clients[i];
        c->id = read_u32(fd);
        c->ready = read_u8(fd);
        deserialize_client_options(fd, &c->options);
    }
}

static void deserialize_round_data(int fd, RoundData *r) {
    r->number = read_u32(fd);
    deserialize_coordinate(fd, &r->answer);
    r->players_len = read_u32(fd);
    r->players = malloc(r->players_len * sizeof(Player));
    for (size_t i = 0; i < r->players_len; i++)
        deserialize_player(fd, &r->players[i]);
}

void deserialize_packet(int fd, Packet *p) {
    p->type = (PacketType)read_u8(fd);
    switch (p->type) {
    case PACKET_INIT:
        deserialize_client_options(fd, &p->data.init.options);
        break;
    case PACKET_CONFIRMED:
        p->data.confirmed.id = read_u32(fd);
        deserialize_client_options(fd, &p->data.confirmed.options);
        deserialize_clients(fd, &p->data.confirmed.lobby);
        break;
    case PACKET_LOBBY_EVENT:
        p->data.lobby_event.action = (LobbyAction)read_u8(fd);
        p->data.lobby_event.user = read_u32(fd);
        deserialize_clients(fd, &p->data.lobby_event.lobby);
        break;
    case PACKET_WAITING_STATUS:
        p->data.waiting_status.ready = read_u8(fd);
        break;
    case PACKET_ROUND_LOADING:
        deserialize_clients(fd, &p->data.round_loading.lobby);
        break;
    case PACKET_ROUND:
        p->data.round.number = read_u32(fd);
        p->data.round.image_len = read_u32(fd);
        read(fd, p->data.round.image, p->data.round.image_len);
        break;
    case PACKET_GUESS:
        deserialize_coordinate(fd, &p->data.guess.coordinates);
        break;
    case PACKET_GUESSED:
        p->data.guessed.player = read_u32(fd);
        break;
    case PACKET_RESULT:
        deserialize_round_data(fd, &p->data.result.round);
        break;
    default:
        break;
    }
}