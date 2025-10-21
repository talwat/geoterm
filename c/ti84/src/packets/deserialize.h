#pragma once
#include "shared.h"

extern LobbyClient LOBBY[16];
extern uint8_t LOBBY_LEN;
extern Player PLAYERS[16];
bool deserialize_packet(Packet *p);
