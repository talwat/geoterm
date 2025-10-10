#include "shared.h"
#include <string.h>
#include <arpa/inet.h>
#include <unistd.h>

static void write_u8(int fd, uint8_t v) { write(fd, &v, 1); }
static void write_u32(int fd, uint32_t v) { v = htonl(v); write(fd, &v, 4); }
static void write_f32(int fd, float f) { write(fd, &f, 4); }

static void serialize_client_options(int fd, const ClientOptions *opt) {
    write_u8(fd, (uint8_t)opt->color);
    write(fd, opt->user, 16);
}

static void serialize_coordinate(int fd, const Coordinate *c) {
    write_f32(fd, c->latitude);
    write_f32(fd, c->longitude);
}

static void serialize_player(int fd, const Player *p) {
    write_u32(fd, (uint32_t)p->id);
    write_u32(fd, (uint32_t)p->points);
    write_u8(fd, p->has_guess);
    if (p->has_guess) serialize_coordinate(fd, &p->guess);
    else { uint8_t pad[2] = {0}; write(fd, pad, 2); }
}

static void serialize_clients(int fd, const Clients *lobby) {
    write_u32(fd, (uint32_t)lobby->len);
    for (size_t i = 0; i < lobby->len; i++) {
        const LobbyClient *c = &lobby->clients[i];
        write_u32(fd, (uint32_t)c->id);
        write_u8(fd, c->ready);
        serialize_client_options(fd, &c->options);
    }
}

static void serialize_round_data(int fd, const RoundData *r) {
    write_u32(fd, (uint32_t)r->number);
    serialize_coordinate(fd, &r->answer);
    write_u32(fd, (uint32_t)r->players_len);
    for (size_t i = 0; i < r->players_len; i++)
        serialize_player(fd, &r->players[i]);
}

void serialize_packet(int fd, const Packet *p) {
    write_u8(fd, (uint8_t)p->type);
    switch (p->type) {
        case PACKET_INIT:
            serialize_client_options(fd, &p->data.init.options);
            break;
        case PACKET_CONFIRMED:
            write_u32(fd, (uint32_t)p->data.confirmed.id);
            serialize_client_options(fd, &p->data.confirmed.options);
            serialize_clients(fd, &p->data.confirmed.lobby);
            break;
        case PACKET_LOBBY_EVENT:
            write_u8(fd, (uint8_t)p->data.lobby_event.action);
            write_u32(fd, (uint32_t)p->data.lobby_event.user);
            serialize_clients(fd, &p->data.lobby_event.lobby);
            break;
        case PACKET_WAITING_STATUS:
            write_u8(fd, p->data.waiting_status.ready);
            break;
        case PACKET_ROUND_LOADING:
            serialize_clients(fd, &p->data.round_loading.lobby);
            break;
        case PACKET_ROUND:
            write_u32(fd, (uint32_t)p->data.round.number);
            write_u32(fd, (uint32_t)p->data.round.image_len);
            write(fd, p->data.round.image, p->data.round.image_len);
            break;
        case PACKET_GUESS:
            serialize_coordinate(fd, &p->data.guess.coordinates);
            break;
        case PACKET_GUESSED:
            write_u32(fd, (uint32_t)p->data.guessed.player);
            break;
        case PACKET_RESULT:
            serialize_round_data(fd, &p->data.result.round);
            break;
        default:
            break;
    }
}
