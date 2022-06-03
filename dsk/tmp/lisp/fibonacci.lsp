(load "/ini/lisp/core.lsp")

(defn fib (n)
  (cond
    ((< n 2) n)
    (true (+ (fib (- n 1)) (fib (- n 2))))))

(println
  (fib
    (cond
      ((null? args) 10)
      (true (parse (car args))))))
