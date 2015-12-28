#include <iostream>
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "memcache_client.h"
#include "tcp_client.h"


int main(int argc, char *argv[]) {
    MemcacheClient cli("127.0.0.1", 11311);
//    cli.printStats();

    std::vector<char> val = {'a', 'b', 'c'};
    assert( true == cli.set("x", val) );

    return 0;
}
