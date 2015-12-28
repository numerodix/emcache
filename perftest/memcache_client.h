#pragma once

#include <iostream>
#include <vector>

#include "tcp_client.h"


class MemcacheClient {
    public:
        MemcacheClient(std::string host, uint16_t port);
        bool get(std::string key, char *data, uint32_t maxlen);
        bool _set(std::string key, const char *data, uint32_t data_len);
        bool set(std::string key, std::vector<char> data);
        void printStats();

    private:
        TcpClient *m_client;
};
