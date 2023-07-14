# MOROS Network

## NET

Display the network configuration:

    > net config
    mac: 52-54-00-12-34-56
    ip:  10.0.2.15/24
    gw:  10.0.2.2
    dns: 10.0.2.3

Display one attribute of the network configuration:

    > net config dns
    dns: 10.0.2.3

Set one attribute of the network configuration:

    > net config dns 10.0.2.3

Display network statistics:

    > net stat
    rx: 13 packets (4052 bytes)
    tx: 15 packets (1518 bytes)

Listen for packets transmitted on the network:

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

The `dhcp` command configures the network automatically:

    > dhcp --verbose
    DEBUG: DHCP Discover transmitted
    DEBUG: DHCP Offer received
    ip:  10.0.2.15/24
    gw:  10.0.2.2
    dns: 10.0.2.3

## HOST

The `host` command performs DNS lookups:

    > host example.com                                                                                 
    example.com has address 93.184.216.34


## TCP

The `tcp` command connects to TCP sockets:

    > tcp time.nist.gov:13 --verbose
    DEBUG: Connecting to 129.6.15.30:13

    58884 20-02-05 19:19:42 00 0 0  49.2 UTC(NIST) *

This could also be done with the `read` command:

    > read /net/tcp/time.nist.gov:13

    58884 20-02-05 19:19:55 00 0 0  49.2 UTC(NIST) *


## HTTP

Requesting a resource on a host:

    > http moros.cc /test.html

Is equivalent to:

    > read /net/http/moros.cc/test.html

And:

    > read /net/http/moros.cc:80/test.html

## SOCKET

The `socket` command is used to read and write to network connexions
like the `netcat` command on Unix.

For example the request made with `tcp` above is equivalent to this:

    > socket time.nist.gov:13 --read-only

    59710 22-05-11 21:44:52 50 0 0 359.3 UTC(NIST) *

And the request made with `http` is equivalent to that:

    > socket moros.cc:80
    GET /test.html HTTP/1.0
    Host: moros.cc

    HTTP/1.1 200 OK
    Server: nginx
    Date: Wed, 11 May 2022 21:46:34 GMT
    Content-Type: text/html
    Content-Length: 866
    Connection: close
    Last-Modified: Fri, 29 Oct 2021 17:50:58 GMT
    ETag: "617c3482-362"
    Accept-Ranges: bytes

    <!doctype html>
    <html>
      <head>
        <meta charset="utf-8">
        <title>MOROS: Obscure Rust Operating System</title>
      </head>
      <body>
        <h1>MOROS</h1>
      </body>
    </html>

Here's a connexion to a SMTP server to send a mail:

    > socket 10.0.2.2:2500
    220 EventMachine SMTP Server
    HELO moros.cc
    250-Ok EventMachine SMTP Server
    MAIL FROM:<vinc@moros.cc>
    250 Ok
    RCPT TO:<alice@example.com>
    250 Ok
    DATA
    354 Send it
    Subject: Test
    Lorem ipsum dolor sit amet, consectetur adipiscing elit. Vestibulum nec
    diam vitae ex blandit malesuada nec a turpis.
    .
    250 Message accepted
    QUIT
    221 Ok

Sending a file to a server:

    > socket 10.0.2.2:1234 <= /tmp/alice.txt
