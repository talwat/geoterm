#pragma once
#include <shared.h>

bool lobby(Packet *packet);
void send_ready(bool ready);
void ready();