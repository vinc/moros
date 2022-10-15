(load "/lib/lisp/core.lsp")

(define (pi-digits y)
  (do
    (define dot true)
    (define q 1)
    (define r 0)
    (define t 1)
    (define k 1)
    (define n 3)
    (define l 3)
    (map
      (lambda (j)
        (do
          (cond
            ((< (- (+ (* q 4) r) t) (* n t)) (do
              (print (string n (cond (dot ".") (true ""))))
              (set dot false)
              (define nr (* 10 (- r (* n t))))
              (set n (- (/ (* 10 (+ (* 3 q) r)) t) (* 10 n)))
              (set q (* q 10))
              (set r nr)))
            (true (do
              (define nr (* (+ (* 2 q) r) l))
              (define nn (/ (+ 2 (* q k 7) (* r l)) (* t l)))
              (set q (* q k))
              (set t (* t l))
              (set l (+ l 2))
              (set k (+ k 1))
              (set n nn)
              (set r nr))))))
      (range 0 y))
    n))

(println
  (cond
    ((nil? args) "Usage: pi <precision>")
    (true (pi-digits (string->number (car args))))))
