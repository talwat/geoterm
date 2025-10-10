#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <termios.h>
#include <unistd.h>

#include "shared.h"

#define SERIAL_DEVICE "/dev/ttyS0"
#define BUF_SIZE 1024

int init_serial(const char *device) {
    int fd = open(device, O_RDWR | O_NOCTTY | O_SYNC);
    if (fd < 0) {
        perror("opening serial device failed");
        exit(EXIT_FAILURE);
    }

    struct termios tty;
    memset(&tty, 0, sizeof tty);
    if (tcgetattr(fd, &tty) != 0) {
        perror("tcgetattr failed");
        exit(EXIT_FAILURE);
    }

    cfsetospeed(&tty, BAUD);
    cfsetispeed(&tty, BAUD);

    tty.c_cflag = (tty.c_cflag & ~CSIZE) | CS8;
    tty.c_iflag &= ~IGNBRK;
    tty.c_lflag = 0;
    tty.c_oflag = 0;
    tty.c_cc[VMIN] = 1;
    tty.c_cc[VTIME] = 0;

    tty.c_iflag &= ~(IXON | IXOFF | IXANY);
    tty.c_cflag |= (CLOCAL | CREAD);
    tty.c_cflag &= ~(PARENB | PARODD);
    tty.c_cflag &= ~CSTOPB;
    tty.c_cflag &= ~CRTSCTS;

    if (tcsetattr(fd, TCSANOW, &tty) != 0) {
        perror("tcsetattr failed");
        exit(EXIT_FAILURE);
    }

    return fd;
}

int init_tcp(char *ip) {
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
    int serial_fd = init_serial(SERIAL_DEVICE);
    int tcp_fd = init_tcp("127.0.0.1");

    char buf[BUF_SIZE];
    ssize_t n;

    fd_set readfds;
    int maxfd = (tcp_fd > serial_fd ? tcp_fd : serial_fd) + 1;

    while (1) {
        FD_ZERO(&readfds);
        FD_SET(tcp_fd, &readfds);
        FD_SET(serial_fd, &readfds);

        if (select(maxfd, &readfds, NULL, NULL, NULL) < 0) {
            perror("select failed");
            break;
        }

        // TCP -> Serial
        if (FD_ISSET(tcp_fd, &readfds)) {
            n = read(tcp_fd, buf, BUF_SIZE);
            if (n <= 0) break;
            write(serial_fd, buf, n);
        }

        // Serial -> TCP
        if (FD_ISSET(serial_fd, &readfds)) {
            n = read(serial_fd, buf, BUF_SIZE);
            if (n <= 0) break;
            write(tcp_fd, buf, n);
        }
    }

    close(tcp_fd);
    close(serial_fd);
    return 0;
}