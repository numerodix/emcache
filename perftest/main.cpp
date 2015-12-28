#include <iostream>
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "memcache_client.h"
#include "tcp_client.h"


int main(int argc, char *argv[]) {
    MemcacheClient cli("127.0.0.1", 11211);
    cli.printStats();

    std::vector<char> val = {'a', 'b', 'c'};

    for (int i=0; i < 100000; i++) {
        assert( true == cli.set("x", val) );
        std::vector<char> val2 = cli.get("x");
        assert( cli.equal(val, val2) );
    }

    return 0;
}
