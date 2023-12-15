# HTTP/0.9 Proxy (https://github.com/vinc/fetch)

(load "/lib/lisp/core.lsp")

(var proxy-host "10.0.2.2")
(var proxy-port 8888)

(var stdout 1)
(var socket (socket/connect "tcp" proxy-host proxy-port))
(file/write socket (str->bin (str "GET " (get args 0) "\n")))
(var open true)
(while open (do
  (var buf (file/read socket 2048))
  (file/write stdout buf)
  (set open (not (nil? buf)))))
