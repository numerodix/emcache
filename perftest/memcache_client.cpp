#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "memcache_client.h"


MemcacheClient::MemcacheClient(string host, uint16_t port) : m_client(nullptr) {
    m_client = new TcpClient(host, port);
}

bool MemcacheClient::get(string key, char *data, uint32_t maxlen) {
    return true;
}

bool MemcacheClient::set(string key, const char *data, uint32_t data_len) {
    assert( key.length() <= 250 );  // memcache upper limit on key length
    assert( data_len <= 1048576 );  // memcache upper limit on value length

    // Construct request
    char header[300] = {0};
    assert( 0 < snprintf(header, 299, "set %s 0 0 %d\r\n", key.c_str(), data_len) );
    uint32_t header_len = strlen(header);

    char line_end[10] = {0};
    assert( 0 < snprintf(line_end, 9, "\r\n") );
    uint32_t line_end_len = strlen(line_end);

    uint32_t request_len = header_len + data_len + line_end_len;
    char request[request_len];
    assert( NULL != memset(request, 0, request_len) );

    assert( NULL != memcpy(request, header, header_len) );
    assert( NULL != memcpy(&request[header_len], data, data_len) );
    assert( NULL != memcpy(&request[header_len + data_len], line_end, line_end_len) );

    // Send request
    m_client->transmit(request, request_len);

    // Receive response
    char response[100] = {0};
    m_client->receive(response, 100);

    // Interpret response
    string resp(response);
    if (resp.compare("STORED\r\n") == 0) {
        return true;
    }

    return false;
}

void MemcacheClient::printStats() {
    string cmd("stats\r\n");
    assert( cmd.length() == m_client->transmit(cmd.c_str(), cmd.length()) );

    char buf[4096];  // 4k is enough stats for everyone
    memset(&buf, 0, 4096);
    assert( 0 < m_client->receive(buf, 4095) );

    cout << buf;
}
