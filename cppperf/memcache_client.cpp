#include <assert.h>
#include <stdio.h>
#include <sstream>
#include <string.h>

#include "memcache_client.h"


MemcacheClient::MemcacheClient(std::string host, uint16_t port) : m_client(nullptr) {
    m_client = new TcpClient(host, port);
}

std::vector<char> MemcacheClient::get(std::string key) {
    // Construct request
    std::stringstream srequest;

    srequest << "get " << key << "\r\n";

    std::string request = srequest.str();

    // Send request
    std::cout << "memcache: Loading key '" << key << "'" << std::endl;
    assert( request.length() == m_client->transmit(request.c_str(), request.length()) );

    // Receive response
    std::vector<char> response;
    char response_buf[4096] = {0};  // 4k is enough stats for everyone
    m_client->receive(response_buf, 4096);
    response.insert(response.end(), &response_buf[0], &response_buf[4096]);

    // Interpret response
    // format: VALUE x 0 3\r\nabc\r\n
    std::string prefix(&response_buf[0], &response_buf[512]);
    // see if it's a successful response
    if (prefix.substr(0, 5).compare("VALUE") == 0) {
        // find the space before the key
        std::size_t pos = prefix.find(" ", 5);

        // find the space before the flags
        pos = prefix.find(" ", pos + 1);

        // find the space before the bytecount
        pos = prefix.find(" ", pos + 1);
        uint32_t data_len = atoi(&prefix[pos]);

        // find the line ending
        pos = prefix.find("\r\n", pos + 1);

        // we know the indices of the data now
        std::vector<char> data(&prefix[pos + 2], &prefix[pos + 2 + data_len]);

        std::string data_str(data.begin(), data.end());
        std::cout << "memcache: Loaded key '" << key << "' with value <<<"
            << data_str << ">>>" << std::endl;

        return data;
    }

    std::cout << "memcache: Failed to load key '" << key << "'" << std::endl;
    std::vector<char> empty;
    return empty;
}

bool MemcacheClient::set(std::string key, std::vector<char> data) {
    // Construct request
    std::stringstream srequest;
    std::string data_str(data.begin(), data.end());

    srequest << "set " << key << " 0 0 " << data.size() << " \r\n";
    srequest << data_str << "\r\n";

    std::string request = srequest.str();

    // Send request
    std::cout << "memcache: Storing key '" << key << "' with value <<<"
        << data_str << ">>>" << std::endl;
    assert( request.length() == m_client->transmit(request.c_str(), request.length()) );

    // Receive response
    char response_buf[101] = {0};
    m_client->receive(response_buf, 100);

    // Interpret response
    std::string response(response_buf);
    if (response.compare("STORED\r\n") == 0) {
        std::cout << "memcache: Stored key '" << key << "'" << std::endl;
        return true;
    }

    std::cout << "memcache: Failed to store key '" << key << "'" << std::endl;
    return false;
}

void MemcacheClient::printStats() {
    // Construct request
    std::string cmd("stats\r\n");

    // Send request
    std::cout << "memcache: Requesting stats" << std::endl;
    assert( cmd.length() == m_client->transmit(cmd.c_str(), cmd.length()) );

    // Receive response
    char buf[4096] = {0};  // 4k is enough stats for everyone
    assert( 0 < m_client->receive(buf, 4095) );
    std::cout << "memcache: Requested stats" << std::endl;

    // Interpret response
    std::string resp(buf);
    std::size_t pos = resp.find("\r\nEND"); // find the end marker
    std::cout << resp.substr(0, pos) << std::endl;
}

bool MemcacheClient::equal(std::vector<char> data1, std::vector<char> data2) {
    return std::equal(data1.begin(), data1.end(), data2.begin());
}
