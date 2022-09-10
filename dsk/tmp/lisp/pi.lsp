(defn pi-nth (n)
  (* (^ 16 (- n)) (-
    (/ 4 (+ 1 (* 8 n)))
    (/ 2 (+ 4 (* 8 n)))
    (/ 1 (+ 5 (* 8 n)))
    (/ 1 (+ 6 (* 8 n))))))

(defn pi-sum (n)
  (apply + (map pi-nth (range 0 n))))
