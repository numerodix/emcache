#pragma once

#include <iostream>


using namespace std;

class TcpClient {
    public:
        TcpClient(string host, uint16_t port);
        bool _connect(); // to avoid clashing with imported symbol 'connect'
        bool transmit(const char* data, uint32_t len);
        uint32_t receive(char *buf, uint32_t len);

    private:
        string m_host;
        uint16_t m_port;
        int32_t m_sockfd;
};
