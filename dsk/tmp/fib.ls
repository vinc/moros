(label fib
  (lambda (n)
    (cond
      ((< n 2) n)
      (true (+ (fib (- n 1)) (fib (- n 2)))))))

(print (fib 6))
