#ifndef SERIALIZE_H
#define SERIALIZE_H

#include "shared.h"

/**
 * Serializes a Packet structure to the given file descriptor.
 *
 * @param fd     The file descriptor to write to (e.g., a socket).
 * @param packet The packet to serialize.
 */
void serialize_packet(int fd, const Packet *packet);

#endif // SERIALIZE_H