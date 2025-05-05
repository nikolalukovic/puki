#ifndef PK_SERVER_H
#define PK_SERVER_H

#include <stddef.h>
#include <stdint.h>

typedef void (*NewConnCallback)(int fd, uint32_t ip, uint16_t port);
typedef void (*DataCallback)(int fd, const uint8_t* data, size_t len);
typedef void (*CloseCallback)(int fd);

int start_server(int port, int event_fd, NewConnCallback new_conn_cb, DataCallback data_cb, CloseCallback close_cb);

#endif
