#pragma once
#include <shared.h>
#include <stdbool.h>

void cleanup();
bool wait(Packet *packet, PacketTag target);