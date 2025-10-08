#include <arpa/inet.h>
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

int main() {
    PacketData data = {.init = {.options = {.color = 1, .user = "tal", .user_len = 3}}};
    Packet packet = {.data = data, .tag = "init"};

    int sock = init_connection("127.0.0.1");

    char buf[128];
    fgets(&buf, sizeof(buf), stdin);

    return 0;
}