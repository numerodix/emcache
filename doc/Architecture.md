# Architecture

                           ----------            ----------
                           | Client |            | Client |
                           ----------            ----------  
                               |                      |
                               |                      |
                         -------------          -------------
                         | Transport |          | Transport |
                         -------------          -------------
                          |    |                      |
    ------------          |    |                      |
    | Listener |          |    \----------\ /---------/
    ------------          |                | 
                          |           ------------
                          |     /-----| Protocol |
    -----------           |    /      ------------
    | Metrics |-----------------------| Storage  |
    -----------                       ------------



## Components

* Storage: Bounded LRU map with single threaded access only, no locking.

* Protocol: Implements the memcached protocol. Performs operations (Cmd) on the storage on behalf of clients and returns reponses (Resp).

* Transport: Converts client requests (in bytes) into Cmd objects and transmits these to the Protocol. Receives Resp objects from the Protocol and writes them out to clients as responses (in bytes).

* Listener: Manages the listening socket and spawns a Transport for each new client. When a client goes away the Transport simply dies (no cleanup is necessary).

* Metrics: Collects server metrics from any other component and aggregates them/displays them.

The Storage and Protocol run in the same thread. All other components run in
separate threads. All communication between threads is done over async channels
(ownership of the sent object is transfered from the sender to the receiver).


## Concepts

* Stats: Part of the memcached protocol. Collected in the Storage, Protocol and Transport. For stats originating in the Transport (eg. bytes sent, bytes received) they are transmitted to (and aggregated at) the Protocol. (Keep in mind that Transports are concurrent, so these stats are always just snapshots and never fully accurate.)

* Metrics: Internal server performance metrics. Any component may collect these and transmit them to the Metrics collector over a channel.
