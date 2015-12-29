#pragma once

#include <iostream>
#include <vector>

#include "tcp_client.h"


class MemcacheClient {
    public:
        MemcacheClient(std::string host, uint16_t port);
        std::vector<char> get(std::string key);
        bool set(std::string key, std::vector<char> data);
        void printStats();

        bool equal(std::vector<char> data1, std::vector<char> data2);

    private:
        TcpClient *m_client;
};
