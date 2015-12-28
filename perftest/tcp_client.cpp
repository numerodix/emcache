#include <arpa/inet.h>
#include <assert.h>
#include <netdb.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <unistd.h>

#include "tcp_client.h"


TcpClient::TcpClient(std::string host, uint16_t port) 
    : m_host(host), m_port(port), m_sockfd(-1) {
}

bool TcpClient::_connect() {
    // Already connected
    if (m_sockfd > -1) {
        return true;
    }

    // Create the socket first
    int32_t sockfd = socket(AF_INET, SOCK_STREAM, 0);
    if (sockfd == -1) {
        perror("Could not create socket");
        return false;
    }

    // Do we need to resolve the hostname?
    struct sockaddr_in server_addr;
    if (inet_addr(m_host.c_str()) == INADDR_NONE) {
        struct hostent *he;
        struct in_addr **addr_list;

        if ((he = gethostbyname(m_host.c_str())) == NULL) {
            perror("gethostbyname() failed");
            std::cout << "tcp: failed to resolve hostname " << m_host << "\n";
            return false;
        }

        addr_list = (struct in_addr**) he->h_addr_list;
        for (int i=0; addr_list[i] != NULL; i++) {
            server_addr.sin_addr = *addr_list[i];
            std::cout << "tcp: " << m_host << " resolved to " 
                << inet_ntoa(*addr_list[i]) << std::endl;
            break;
        }

    // We have an ip address already
    } else {
        server_addr.sin_addr.s_addr = inet_addr(m_host.c_str());
    }

    server_addr.sin_family = AF_INET;
    server_addr.sin_port = htons(m_port);

    // Establish connection
    if (connect(sockfd, (struct sockaddr*) &server_addr, sizeof(server_addr)) < 0) {
        perror("connect() failed");
        return false;
    }

    std::cout << "tcp: connected to " << m_host << ":" << m_port << "\n";
    m_sockfd = sockfd;
    return true;
}

uint32_t TcpClient::transmit(const char* data, uint32_t len) {
    assert( this->_connect() == true );

    ssize_t bytes_cnt = send(m_sockfd, data, strlen(data), 0);
    if (bytes_cnt < 0) {
        perror("send() failed");
    } else {
        std::cout << "tcp: sent " << bytes_cnt << " bytes\n";
    }

    return (uint32_t) bytes_cnt;
}

uint32_t TcpClient::receive(char* data, uint32_t len) {
    assert( this->_connect() == true );

    ssize_t bytes_cnt = recv(m_sockfd, data, len, 0);
    if (bytes_cnt < 0) {
        perror("recv() failed");
    } else {
        std::cout << "tcp: received " << bytes_cnt << " bytes\n";
    }

    return (uint32_t) bytes_cnt;
}
