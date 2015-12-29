#CXX := g++
CXX := clang++-3.6
CXX_FLAGS := -std=c++11 -Wall -pedantic

all: perftest


perftest: main.cpp memcache_client.o tcp_client.o
	$(CXX) $(CXX_FLAGS) main.cpp memcache_client.o tcp_client.o -o $@

memcache_client.o: memcache_client.h memcache_client.cpp tcp_client.o
	$(CXX) $(CXX_FLAGS) -c memcache_client.cpp -o $@

tcp_client.o: tcp_client.h tcp_client.cpp
	$(CXX) $(CXX_FLAGS) -c tcp_client.cpp -o $@


clean:
	-rm perftest *.o
