#ifndef SERIALIZE_H
#define SERIALIZE_H

#include "shared.h"

const

    /**
     * Serializes a Packet structure to the given file descriptor.
     * This function writes the binary representation of the packet
     * in the same format as the Rust serializer.
     *
     * @param fd     The file descriptor to write to (e.g., a socket).
     * @param packet The packet to serialize.
     */
    void
    serialize_packet(int fd, const Packet *packet);

#endif // SERIALIZE_H