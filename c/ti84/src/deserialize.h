#ifndef DESERIALIZE_H
#define DESERIALIZE_H

#include "shared.h"

/**
 * Deserializes a Packet structure from the given file descriptor.
 * The function blocks until a full packet is read from the stream.
 *
 * The resulting Packet may allocate heap memory for nested data
 * such as Clients and RoundData. The caller is responsible for
 * freeing these using `free_packet()`.
 *
 * @param fd     The file descriptor to read from.
 * @param packet Pointer to a Packet structure to fill.
 */
void deserialize_packet(int fd, Packet *packet);

/**
 * Frees all heap allocations inside a Packet.
 * This should be called after processing a received Packet
 * to avoid memory leaks.
 *
 * @param packet The packet to free.
 */
void free_packet(Packet *packet);

#endif // DESERIALIZE_H
