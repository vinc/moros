(load "/lib/lisp/core.lsp")
(set mtu (file/size "/dev/net/tcp"))
(set port 80)
(print (str "Listening to 0.0.0.0:" port))
(set socket (socket/listen "tcp" port))
(while true (do
  (set addr (socket/accept socket))
  (if (not (nil? addr)) (do
    (print (str "Connection from " addr))
    (set buf (file/read socket mtu))
    (file/write socket buf)
    (file/close socket)
    (set socket (socket/listen "tcp" port))))))
