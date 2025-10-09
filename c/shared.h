#ifndef SHARED_H
#define SHARED_H

#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

#define PORT 4000
#define IMAGE_W 320
#define IMAGE_H 240
#define IMAGE_SIZE (IMAGE_W * IMAGE_H)

typedef enum { LOBBY_JOIN = 0, LOBBY_RETURN, LOBBY_LEAVE, LOBBY_READY } LobbyAction;
typedef enum { COLOR_RED = 0, COLOR_YELLOW, COLOR_GREEN, COLOR_BLUE, COLOR_MAGENTA } Color;

typedef struct {
    float lon;
    float lat;
} Coordinate;

typedef struct {
    char *user;
    size_t user_len;
    Color color;
} ClientOptions;

typedef struct {
    size_t id;
    bool ready;
    ClientOptions options;
} LobbyClient;

typedef struct {
    bool has_guess;
    Coordinate guess;
    uint64_t points;
    size_t id;
} Player;

typedef struct {
    size_t number;
    Coordinate answer;
    Player *players;
    size_t players_len;
} RoundData;

typedef union {
    struct {
        ClientOptions options;
    } init;
    struct {
        size_t id;
        ClientOptions options;
        LobbyClient *lobby;
        size_t lobby_len;
    } confirmed;
    struct {
        LobbyAction action;
        size_t user;
        LobbyClient *lobby;
        size_t lobby_len;
    } lobby_event;
    struct {
        bool ready;
    } waiting_status;
    struct {
        LobbyClient *lobby;
        size_t lobby_len;
    } round_loading;
    struct {
        size_t number;
        uint8_t image[IMAGE_SIZE];
    } round_image;
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
    const char *tag;
    PacketData data;
} Packet;

#endif // SHARED_H