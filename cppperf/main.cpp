#include <chrono>
#include <iostream>
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "memcache_client.h"
#include "tcp_client.h"


int main(int argc, char *argv[]) {
    MemcacheClient cli("127.0.0.1", 11311);
    cli.printStats();

    uint32_t num_requests = 10000;
    std::vector<char> val = {'a', 'b', 'c'};

    auto begin = std::chrono::steady_clock::now();
    for (int i=0; i < num_requests; i++) {
        assert( true == cli.set("x", val) );
        std::vector<char> val2 = cli.get("x");
        assert( cli.equal(val, val2) );
    }
    auto end = std::chrono::steady_clock::now();
    auto dur_ms = std::chrono::duration_cast<std::chrono::milliseconds>(end - begin).count();
    double dur = (double) dur_ms / 1000;
    double rate = num_requests / dur;

    std::cout << "Made " << num_requests << " constant key set+get requests in " << dur <<
        " seconds = " << rate << " requests/sec" << std::endl;

    return 0;
}
