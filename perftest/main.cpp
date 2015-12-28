#include <iostream>
#include <assert.h>
#include <stdio.h>
#include <string.h>

#include "tcp_client.h"


using namespace std;

int main(int argc, char *argv[]) {
    TcpClient cli("127.0.0.1", 11211);
    assert( true == cli._connect() );

    string cmd("stats\r\n");
    assert( true == cli.transmit(cmd.c_str(), cmd.length()) );

    char buf[4096];
    memset(&buf, 0, 4097);
    assert( 0 < cli.receive(buf, 4096) );

    printf("got: %s\n", buf);

    return 0;
}
