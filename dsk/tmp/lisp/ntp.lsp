(load "/lib/lisp/core.lsp")

(var addr (get args 0)) # Run `host pool.ntp.org` to get an address
(var port 123)
(var socket (socket.connect "udp" addr port))

(var req (map (fun (i) (if (eq? i 0) 0x33 0)) (range 0 48)))
(file.write socket req)
(var res (file.read socket 48))

(var buf (slice res 40 4))
(var time (- (bin->num (concat '(0 0 0 0) buf) "int") 2208988800))
(print time)
