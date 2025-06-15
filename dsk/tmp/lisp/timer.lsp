(load "/lib/lisp/core.lsp")

(var bar-width 67)

(def (repeat ch n) (do
  (var s "")
  (var i 0)
  (while (< i n) (do
    (set s (str s ch))
    (set i (+ i 1))))
  s))

(def (parse-duration d) (do
  (var cs (chars d))
  (var ns nil)
  (var n 0)
  (while (not (empty? cs)) (do
    (var c (first cs))
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
  (var filled (trunc (* (/ elapsed total) bar-width)))
  (var empty  (- bar-width filled))
  (str "[\e[92m" (repeat "#" filled) "\e[0m" (repeat "-" empty) "]")))

(def (format-time secs) (do
  (var m (trunc (/ secs 60)))
  (var s (trunc (rem secs 60)))
  (var mm (if (< m 10) (str "0" (num->str m)) (num->str m)))
  (var ss (if (< s 10) (str "0" (num->str s)) (num->str s)))
  (str mm ":" ss)))

(def (timer label duration) (do
  (var start (clock/epoch))
  (while (< (clock/epoch) (+ start duration)) (do
    (var elapsed (- (clock/epoch) start))
    (var remaining (- duration elapsed))
    (p (str "\e[2K\e[1G" label " "
      (progress-bar elapsed duration) " "
      (format-time remaining)))
    (sleep 1)))
  (print (str "\e[2K\e[1G" label " "
    (progress-bar duration duration) " "
    (format-time 0)))))

(var l (if (> (len args) 0) (get args 0) "Wait"))
(var d (if (> (len args) 1) (parse-duration (get args 1)) 60))

(timer l d)
