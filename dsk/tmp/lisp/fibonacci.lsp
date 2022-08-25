(load "/ini/lisp/core.lsp")

(defn fib (n)
  (cond
    ((< n 2) n)
    (true (+ (fib (- n 1)) (fib (- n 2))))))

(println
  (cond
    ((null? args) "Usage: fibonacci <num>")
    (true (fib (parse (car args))))))
