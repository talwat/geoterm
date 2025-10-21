#ifndef SHARED_H
#define SHARED_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define PORT 4000
#define BAUD 9600 * 4
#define IMAGE_W 320
#define IMAGE_H 240

typedef enum { LOBBY_JOIN = 0, LOBBY_RETURN, LOBBY_LEAVE, LOBBY_READY } LobbyAction;
typedef enum { RED = 0, GREEN, BLUE, MAGENTA } Color;
typedef enum {
    PACKET_INIT = 1,
    PACKET_CONFIRMED,
    PACKET_LOBBY_EVENT,
    PACKET_WAITING_STATUS,
    PACKET_ROUND_LOADING,
    PACKET_ROUND,
    PACKET_GUESS,
    PACKET_GUESSED,
    PACKET_RESULT,
    PACKET_REQUEST_GAME_END
} PacketTag;

typedef struct {
    float longitude;
    float latitude;
} Coordinate;

typedef struct {
    Color color;
    char user[16];
} ClientOptions;

typedef struct {
    size_t id;
    bool ready;
    ClientOptions options;
} LobbyClient;

typedef struct {
    size_t len;
    LobbyClient *clients;
} Clients;

typedef struct {
    bool has_guess;
    Coordinate guess;
    uint32_t points;
    uint32_t delta;
    size_t id;
} Player;

typedef struct {
    size_t number;
    Coordinate answer;
    size_t players_len;
    Player *players;
} RoundData;

typedef union {
    struct {
        ClientOptions options;
    } init;
    struct {
        size_t id;
        ClientOptions options;
        Clients lobby;
    } confirmed;
    struct {
        LobbyAction action;
        size_t user;
        Clients lobby;
    } lobby_event;
    struct {
        bool ready;
    } waiting_status;
    struct {
        Clients lobby;
    } round_loading;
    struct {
        size_t number;
        size_t image_len;
        unsigned char *image;
    } round;
    struct {
        Coordinate coordinates;
    } guess;
    struct {
        size_t player;
    } guessed;
    struct {
        RoundData round;
    } result;
    struct {
    } return_to_lobby;
} PacketData;

typedef struct {
    PacketTag tag;
    PacketData data;
} Packet;

#endif // SHARED_H