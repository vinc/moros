(load "/lib/lisp/core.lsp")

(def (pi-digits digits)
  (do
    (def i 0)
    (def q 1)
    (def r 0)
    (def t 1)
    (def k 1)
    (def n 3)
    (def l 3)
    (while (<= i digits)
      (if (< (- (+ (* q 4) r) t) (* n t))
        (do
          (print (string n (if (= i 0) "." "")))
          (set i (+ i 1))
          (def nr (* 10 (- r (* n t))))
          (set n (- (/ (* 10 (+ (* 3 q) r)) t) (* 10 n)))
          (set q (* q 10))
          (set r nr))
        (do
          (def nr (* (+ (* 2 q) r) l))
          (def nn (/ (+ 2 (* q k 7) (* r l)) (* t l)))
          (set q (* q k))
          (set t (* t l))
          (set l (+ l 2))
          (set k (+ k 1))
          (set n nn)
          (set r nr))))
    ""))

(println
  (if (nil? args) "Usage: pi <precision>"
    (pi-digits (string->number (car args)))))
