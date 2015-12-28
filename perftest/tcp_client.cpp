#include <iostream>
#include <arpa/inet.h>
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>


using namespace std;

class TcpClient {
    public:
        TcpClient(string host, uint16_t port);
        bool connect();
        uint32_t send(const char* data, uint32_t len);
        uint32_t recv(char *buf, uint32_t len);

    private:
        string m_host;
        uint16_t m_port;
        uint16_t m_socket;
};

TcpClient::TcpClient(string host, uint16_t port) 
    : m_host(host), m_port(port), m_socket(-1) {
}

bool TcpClient::connect() {
    // Already connected
    if (m_socket > -1) {
        return true;
    }

    uint32_t sock = socket(AF_INET, SOCK_STREAM, 0);
    if (sock == -1) {
        perror("Could not create socket");
        return false;
    }
}


int main() {
    TcpClient cli("127.0.0.1", 11211);
    assert( true == cli.connect() );

    string cmd("stats\r\n");
    assert( cmd.length() == cli.send(cmd.c_str(), cmd.length()) );

    char buf[101];
    memset(buf, 0, 101);
    assert( 0 > cli.recv(buf, 100) );

    printf("got: %s", buf);

    return 0;
}
