CC = gcc
CFLAGS = -Wall -Wextra -O2 -I.. -I../cmp

TARGET = transponder
SRC = main.c

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) -o $@ $^

clean:
	rm -f $(TARGET)