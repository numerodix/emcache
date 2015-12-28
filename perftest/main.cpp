#include <iostream>
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "memcache_client.h"
#include "tcp_client.h"


using namespace std;

int main(int argc, char *argv[]) {
    MemcacheClient cli("127.0.0.1", 11311);
//    cli.printStats();

    assert( true == cli.set("x", "abc", 3) );

    return 0;
}
