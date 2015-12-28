#pragma once

#include <iostream>
#include <vector>

#include "tcp_client.h"


using namespace std;

class MemcacheClient {
    public:
        MemcacheClient(string host, uint16_t port);
        bool get(string key, char *data, uint32_t maxlen);
        bool _set(string key, const char *data, uint32_t data_len);
        bool set(string key, vector<char> data);
        void printStats();

    private:
        TcpClient *m_client;
};
