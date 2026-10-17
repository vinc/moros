(load "/lib/lisp/core.lsp")

(set bar-width 67)

(def (repeat ch n) (do
  (set s "")
  (set i 0)
  (while (< i n) (do
    (set s (str s ch))
    (set i (+ i 1))))
  s))

(def (parse-duration d) (do
  (set cs (chars d))
  (set ns nil)
  (set n 0)
  (while (not (empty? cs)) (do
    (set c (first cs))
    (set cs (rest cs))
    (if (contains? "0123456789" c)
      (set n (+ (* n 10) (str->num c)))
      (do
        (if (contains? "hms" c)
          (if (eq? c "h")
            (set n (* n 3600))
            (if (eq? c "m")
              (set n (* n 60)))))
        (set ns (push ns n))
        (set n 0)))))
  (set ns (push ns n))
  (reduce + ns)))

(def (progress-bar elapsed total) (do
  (set filled (trunc (* (/ elapsed total) bar-width)))
  (set empty  (- bar-width filled))
  (str "[\e[92m" (repeat "#" filled) "\e[0m" (repeat "-" empty) "]")))

(def (format-time secs) (do
  (set m (trunc (/ secs 60)))
  (set s (trunc (% secs 60)))
  (set mm (if (< m 10) (str "0" (num->str m)) (num->str m)))
  (set ss (if (< s 10) (str "0" (num->str s)) (num->str s)))
  (str mm ":" ss)))

(def (timer label duration) (do
  (set start (clock/epoch))
  (while (< (clock/epoch) (+ start duration)) (do
    (set elapsed (- (clock/epoch) start))
    (set remaining (- duration elapsed))
    (p (str "\e[2K\e[1G" label " "
      (progress-bar elapsed duration) " "
      (format-time remaining)))
    (sleep 1)))
  (print (str "\e[2K\e[1G" label " "
    (progress-bar duration duration) " "
    (format-time 0)))))

(set l (if (> (len args) 0) (get args 0) "Wait"))
(set d (if (> (len args) 1) (parse-duration (get args 1)) 60))

(timer l d)
