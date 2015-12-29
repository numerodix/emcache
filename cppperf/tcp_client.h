#pragma once

#include <iostream>
#include <stdint.h>


class TcpClient {
    public:
        TcpClient(std::string host, uint16_t port);
        bool _connect(); // to avoid clashing with imported symbol 'connect'
        uint32_t transmit(const char* data, uint32_t len);
        uint32_t receive(char *buf, uint32_t len);

    private:
        std::string m_host;
        uint16_t m_port;
        int32_t m_sockfd;
};
