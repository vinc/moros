(load "/lib/lisp/core.lsp")

(def (pi-digits digits)
  (do
    (set i 0)
    (set q 1)
    (set r 0)
    (set t 1)
    (set k 1)
    (set n 3)
    (set l 3)
    (while (<= i digits)
      (if (< (- (+ (* q 4) r) t) (* n t))
        (do
          (p (str n (if (= i 0) "." "")))
          (set i (+ i 1))
          (set nr (* 10 (- r (* n t))))
          (set n (- (/ (* 10 (+ (* 3 q) r)) t) (* 10 n)))
          (set q (* q 10))
          (set r nr))
        (do
          (set nr (* (+ (* 2 q) r) l))
          (set nn (/ (+ 2 (* q k 7) (* r l)) (* t l)))
          (set q (* q k))
          (set t (* t l))
          (set l (+ l 2))
          (set k (+ k 1))
          (set n nn)
          (set r nr))))
    ""))

(print
  (if (nil? args) "Usage: pi <precision>"
    (pi-digits (str->num (head args)))))
