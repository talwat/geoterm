#ifndef DESERIALIZE_H
#define DESERIALIZE_H

#include "shared.h"

/**
 * Deserializes a Packet structure from the given file descriptor.
 * The function blocks until a full packet is read from the stream.

 * @param fd     The file descriptor to read from.
 * @param packet Pointer to a Packet structure to fill.
 */
void deserialize_packet(int fd, Packet *packet);

#endif // DESERIALIZE_H
