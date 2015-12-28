#CXX := g++
CXX := clang++-3.6
CXX_FLAGS := -std=c++11 -Wall -pedantic

all: perftest


perftest: tcp_client.cpp
	$(CXX) $(CXX_FLAGS) tcp_client.cpp -o perftest


clean:
	-rm perftest
