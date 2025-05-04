#include "server.h"

#include <arpa/inet.h>
#include <errno.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/epoll.h>
#include <sys/eventfd.h>
#include <sys/socket.h>
#include <unistd.h>

#define MAX_EVENTS 10
#define BUFFER_SIZE 1024

static NewConnCallback new_conn_cb = NULL;
static DataCallback data_cb = NULL;
static CloseCallback close_cb = NULL;

static void close_client(int client_fd, int epoll_fd) {
  struct sockaddr_in peer_addr;
  socklen_t peer_len = sizeof(peer_addr);
  char peer_ip[INET_ADDRSTRLEN];
  int peer_port = 0;

  if (getpeername(client_fd, &peer_addr, &peer_len) == 0) {
    inet_ntop(AF_INET, &peer_addr.sin_addr, peer_ip, INET_ADDRSTRLEN);
    peer_port = ntohs(peer_addr.sin_port);
  }

  printf("Client disconnected: %s:%d (fd: %d)\n", peer_ip, peer_port,
         client_fd);

  if (close_cb) {
    close_cb(client_fd);
  }

  epoll_ctl(epoll_fd, EPOLL_CTL_DEL, client_fd, NULL);
  close(client_fd);
}

int start_server(int port, int shutdown_event_fd,
                 NewConnCallback new_conn_cb_in, DataCallback data_cb_in,
                 CloseCallback close_cb_in) {
  int listen_sock;
  int epoll_fd;
  struct sockaddr_in server_addr;
  struct epoll_event ev;
  struct epoll_event events[MAX_EVENTS];
  bool running = true;
  char buffer[BUFFER_SIZE];
  new_conn_cb = new_conn_cb_in;
  data_cb = data_cb_in;
  close_cb = close_cb_in;

  listen_sock = socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0);
  if (listen_sock < 0) {
    perror("socket");
    return -1;
  }

  int optval = 1;
  setsockopt(listen_sock, SOL_SOCKET, SO_REUSEADDR, &optval, sizeof(optval));

  memset(&server_addr, 0, sizeof(server_addr));
  server_addr.sin_family = AF_INET;
  server_addr.sin_addr.s_addr = htonl(INADDR_ANY);
  server_addr.sin_port = htons(port);

  if (bind(listen_sock, (struct sockaddr *)&server_addr, sizeof(server_addr)) <
      0) {
    perror("bind");
    close(listen_sock);
    return -1;
  }

  if (listen(listen_sock, SOMAXCONN) < 0) {
    perror("listen");
    close(listen_sock);
    return -1;
  }

  epoll_fd = epoll_create1(0);
  if (epoll_fd < 0) {
    perror("epoll_create1");
    close(listen_sock);
    return -1;
  }

  ev.events = EPOLLIN;
  ev.data.fd = listen_sock;
  if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, listen_sock, &ev) < 0) {
    perror("epoll_ctl");
    close(epoll_fd);
    close(listen_sock);
    return -1;
  }

  ev.events = EPOLLIN;
  ev.data.fd = shutdown_event_fd;
  if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, shutdown_event_fd, &ev) < 0) {
    perror("epoll_ctl");
    close(epoll_fd);
    close(listen_sock);
    return -1;
  }

  while (running) {
    int nfds = epoll_wait(epoll_fd, events, MAX_EVENTS, -1);
    if (nfds < 0) {
      if (errno == EINTR)
        continue;

      perror("epoll_wait");
      break;
    }

    for (int i = 0; i < nfds; ++i) {
      int current_fd = events[i].data.fd;
      uint32_t current_events = events[i].events;

      if (current_fd == shutdown_event_fd) {
        uint64_t event_val;
        read(shutdown_event_fd, &event_val, sizeof(uint64_t));
        running = 0;
        break;
      } else if (current_fd == listen_sock) {
        while (true) {
          struct sockaddr_in client_addr;
          socklen_t client_len = sizeof(client_addr);
          char client_ip[INET_ADDRSTRLEN];

          int conn_sock = accept4(listen_sock, (struct sockaddr *)&client_addr,
                                  &client_len, SOCK_NONBLOCK);
          if (conn_sock < 0) {
            if (errno == EAGAIN || errno == EWOULDBLOCK) {
              break;
            } else {
              perror("accept4");
              break;
            }
          }

          getpeername(conn_sock, &client_addr, &client_len);
          inet_ntop(AF_INET, &client_addr.sin_addr, client_ip, INET_ADDRSTRLEN);

          printf("New connection from %s:%d\n", client_ip,
                 ntohs(client_addr.sin_port));

          ev.events = EPOLLIN | EPOLLRDHUP;
          ev.data.fd = conn_sock;
          if (epoll_ctl(epoll_fd, EPOLL_CTL_ADD, conn_sock, &ev) < 0) {
            perror("epoll_ctl");
            close(conn_sock);
          }
          if (new_conn_cb != NULL) {
            new_conn_cb(conn_sock, client_addr.sin_addr.s_addr,
                        client_addr.sin_port);
          }
        }
      } else {
        int client_fd = current_fd;

        if ((current_events & EPOLLERR) || (current_events & EPOLLHUP) ||
            (current_events & EPOLLRDHUP)) {
          close_client(client_fd, epoll_fd);
          continue;
        }

        if (current_events & EPOLLIN) {
          ssize_t bytes_read = read(client_fd, buffer, BUFFER_SIZE - 1);
          if (bytes_read > 0) {
            struct sockaddr_in peer_addr;
            socklen_t peer_len = sizeof(peer_addr);
            char peer_ip[INET_ADDRSTRLEN];
            int peer_port = 0;

            if (getpeername(client_fd, &peer_addr, &peer_len) == 0) {
              inet_ntop(AF_INET, &peer_addr.sin_addr, peer_ip, INET_ADDRSTRLEN);
              peer_port = ntohs(peer_addr.sin_port);
            } else {
              strncpy(peer_ip, "unknown", INET_ADDRSTRLEN);
            }

            printf("Got %zd bytes from %s:%d: %.*s\n", bytes_read, peer_ip,
                   peer_port, (int)bytes_read, buffer);

            if (data_cb != NULL) {
              data_cb(client_fd, (const uint8_t *)buffer, bytes_read);
            }

            ssize_t bytes_written = write(client_fd, buffer, bytes_read);
            if (bytes_written < 0) {
              if (errno == EAGAIN || errno == EWOULDBLOCK) {
              } else {
                perror("C: write failed");
                close_client(client_fd, epoll_fd);
              }
            } else if (bytes_written < bytes_read) {
            }
          } else if (bytes_read == 0) {
            close_client(client_fd, epoll_fd);
          } else if (bytes_read < 0) {
            if (errno == EAGAIN || errno == EWOULDBLOCK) {

            } else {
              perror("read");
              close_client(client_fd, epoll_fd);
            }
          }
        }
      }
    }
  }

  close(listen_sock);
  close(epoll_fd);

  return 0;
}
