# MOROS Network

## NET

The `net` command allows you to configure your network interface:

    > net config debug true

And listen what is happening on the network:

    > net monitor
    ------------------------------------------------------------------
    [488.396667] NET RTL8139 Receiving:
    00000000: 3333 0000 0001 5256 0000 0002 86DD 6000 33....RV......`.
    00000010: 0000 0038 3AFF FE80 0000 0000 0000 0000 ...8:...........
    00000020: 0000 0000 0002 FF02 0000 0000 0000 0000 ................
    00000030: 0000 0000 0001 8600 155E 4000 0708 0000 .........^@.....
    00000040: 0000 0000 0000 0101 5256 0000 0002 0304 ........RV......
    00000050: 40C0 0001 5180 0000 3840 0000 0000 FEC0 @...Q...8@......
    00000060: 0000 0000 0000 0000 0000 0000 0000      ..............
    ------------------------------------------------------------------
    [543.871322] NET RTL8139 Receiving:
    00000000: 5254 0012 3456 5255 0A00 0202 0800 4500 RT..4VRU .....E.
    00000010: 002C 0001 0000 4006 62BB 0A00 0202 0A00 .,....@.b. ... .
    00000020: 020F A2E8 0016 0412 F801 0000 0000 6002 ..............`.
    00000030: 2238 BECB 0000 0204 05B4 0000           "8..........
    ------------------------------------------------------------------

## DHCP

The `dhcp` command configures your network automatically:

    > dhcp
    DHCP Discover transmitted
    DHCP Offer received
    Leased: 10.0.2.15/24
    Router: 10.0.2.2
    DNS: 10.0.2.3

## IP

The `ip` command displays information about your IP address:

    > ip
    Link: 52-54-00-12-34-56
    Addr: 10.0.2.15/24
    RX packets: 1
    TX packets: 1
    RX bytes: 590
    TX bytes: 299

It can also be used to set your IP address:

    > ip set 10.0.2.15/24

## ROUTE

The `route` command displays the IP routing table:

    > route
    Destination         Gateway
    0.0.0.0/0           10.0.2.2

NOTE: It will later allow you to manipulate it.

## HOST

The `host` command performs DNS lookups:

    > host example.com                                                                                 
    example.com has address 93.184.216.34


## TCP

The `tcp` command connects to TCP sockets:

    > tcp time.nist.gov 13
    Connecting to 129.6.15.30:13

    58884 20-02-05 19:19:42 00 0 0  49.2 UTC(NIST) *

This could also be done with the `read` command:

    > read /net/tcp/time.nist.gov:13
    Connecting to 129.6.15.30:13

    58884 20-02-05 19:19:55 00 0 0  49.2 UTC(NIST) *


## HTTP

Requesting a resource on a host:

    > http example.com /articles/index.html

Is equivalent to:

    > read /net/http/example.com/articles

And:

    > read /net/http/example.com:80/articles/index.html
