(load "/lib/lisp/core.lsp")

(def (equation-of-time y)
  (* 60.0 229.18 (+ 0.000075 (-
    (* 0.001868 (cos (* 1.0 y)))
    (* 0.032077 (sin (* 1.0 y)))
    (* 0.014615 (cos (* 2.0 y)))
    (* 0.040849 (sin (* 2.0 y)))))))

(def (days timestamp)
  (trunc (% (/ timestamp 86400.0) 365.2425)))

(def (hours timestamp)
  (trunc (/ (% timestamp 86400.0) 3600.0)))

(def (seconds timestamp longitude)
  (+
    (% timestamp 86400.0)
    (/ (* longitude 86400.0) 360.0)
    (equation-of-time (*
      (/ (* 2 pi) 365.0)
      (+ (days timestamp) (/ (- (hours timestamp) 12.0) 24.0))))))

(def (pad x)
  (str (if (< x 10) "0" "") x))

(def (fmt x)
  (str (pad (trunc x)) ":" (pad (abs (trunc (* (- x (trunc x)) 100.0))))))

(def (geotime longitude timestamp)
  (fmt (% (/ (* (seconds timestamp longitude) 100.0) 86400.0) 100.0)))

(print
  (if (= (len args) 1)
    (geotime (str->num (first args)) (clk/epoch))
    (if (= (len args) 2)
      (geotime (str->num (first args)) (str->num (second args)))
      "Usage: geotime <longitude> [<timestamp>]")))
