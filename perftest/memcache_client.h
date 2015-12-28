#pragma once

#include <iostream>

#include "tcp_client.h"


using namespace std;

class MemcacheClient {
    public:
        MemcacheClient(string host, uint16_t port);
        bool get(string key, char *data, uint32_t maxlen);
        bool set(string key, const char *data, uint32_t data_len);
        void printStats();

    private:
        TcpClient *m_client;
};
