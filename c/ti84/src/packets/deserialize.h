#ifndef DESERIALIZE_H
#define DESERIALIZE_H

#include "shared.h"

extern LobbyClient LOBBY[16];
extern uint8_t LOBBY_LEN;
extern Player PLAYERS[16];
bool deserialize_packet(Packet *p);

#endif // DESERIALIZE_H
