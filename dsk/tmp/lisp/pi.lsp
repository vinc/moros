(load "/lib/lisp/core.lsp")

(def (pi-digits digits)
  (do
    (var i 0)
    (var q 1)
    (var r 0)
    (var t 1)
    (var k 1)
    (var n 3)
    (var l 3)
    (while (<= i digits)
      (if (< (- (+ (* q 4) r) t) (* n t))
        (do
          (p (str n (if (= i 0) "." "")))
          (set i (+ i 1))
          (var nr (* 10 (- r (* n t))))
          (set n (- (/ (* 10 (+ (* 3 q) r)) t) (* 10 n)))
          (set q (* q 10))
          (set r nr))
        (do
          (var nr (* (+ (* 2 q) r) l))
          (var nn (/ (+ 2 (* q k 7) (* r l)) (* t l)))
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
