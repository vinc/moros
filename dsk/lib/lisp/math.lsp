(def (floor x)
  "Returns the largest integer less than or equal to the number"
  (if (>= x 0.0)
    (trunc x)
    (if (= x (trunc x))
      (trunc x)
      (- (trunc x) 1))))

(def (ceil x)
  "Returns the smallest integer greater than or equal to the number"
  (if (<= x 0.0)
    (trunc x)
    (if (= x (trunc x))
      (trunc x)
      (+ (trunc x) 1))))

(def (round x)
  "Returns the nearest integer to the number"
  (let (a b) ((floor x) (ceil x))
    (if (< (- x a) 0.5) a b)))
