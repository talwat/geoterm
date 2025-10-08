#include <arpa/inet.h>
#include <cmp.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "shared.h"

int init_connection(char *ip) {
    int fd;
    struct sockaddr_in address;
    socklen_t address_len = sizeof(address);

    if ((fd = socket(AF_INET, SOCK_STREAM, 0)) < 0) {
        perror("error: socket failed\n");
        exit(EXIT_FAILURE);
    }

    address.sin_family = AF_INET;
    address.sin_addr.s_addr = inet_addr(ip);
    address.sin_port = htons(PORT);

    if (connect(fd, (struct sockaddr *)&address, address_len) < 0) {
        perror("error: connection failed\n");
        exit(EXIT_FAILURE);
    } else {
        printf("client: connected to server on port %d\n", PORT);
    }

    return fd;
}

void send_init_packet(int sock, Packet *packet) {
    cmp_ctx_t cmp;
    // TODO: Make the IO functionality. No duh.
    cmp_init(&cmp, &sock, NULL, NULL, NULL);

    // root map: {"tag": string, "data": {...}}
    cmp_write_map(&cmp, 2);

    // tag
    cmp_write_str(&cmp, "tag", 3);
    cmp_write_str(&cmp, packet->tag, strlen(packet->tag));

    // data
    cmp_write_str(&cmp, "data", 4);
    cmp_write_map(&cmp, 1);

    cmp_write_str(&cmp, "options", 7);
    cmp_write_map(&cmp, 2);

    cmp_write_str(&cmp, "user", 4);
    cmp_write_str(&cmp, packet->data.init.options.user, packet->data.init.options.user_len);

    cmp_write_str(&cmp, "color", 5);
    cmp_write_u8(&cmp, packet->data.init.options.color);
}

int main() {
    PacketData data = {.init = {.options = {.color = 1, .user = "tal", .user_len = 3}}};

    Packet packet = {.data = data, .tag = "init"};

    int sock = init_connection("127.0.0.1");
    send_init_packet(sock, &packet);

    char buf[128];
    fgets(&buf, sizeof(buf), stdin);

    return 0;
}