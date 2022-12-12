(load "/lib/lisp/core.lsp")

(define (equation-of-time y)
  (* 60.0 229.18 (+ 0.000075 (-
    (* 0.001868 (cos (* 1.0 y)))
    (* 0.032077 (sin (* 1.0 y)))
    (* 0.014615 (cos (* 2.0 y)))
    (* 0.040849 (sin (* 2.0 y)))))))

(define (days timestamp)
  (trunc (% (/ timestamp 86400.0) 365.2425)))

(define (hours timestamp)
  (trunc (/ (% timestamp 86400.0) 3600.0)))

(define (seconds timestamp longitude)
  (+
    (% timestamp 86400.0)
    (/ (* longitude 86400.0) 360.0)
    (equation-of-time (*
      (/ (* 2 pi) 365.0)
      (+ (days timestamp) (/ (- (hours timestamp) 12.0) 24.0))))))

(define (abs x)
  (if (< x 0) (- x) x))

(define (pad x)
  (string (if (< x 10) "0" "") x))

(define (fmt x)
  (string (pad (trunc x)) ":" (pad (abs (trunc (* (- x (trunc x)) 100.0))))))

(define (geotime longitude timestamp)
  (fmt (% (/ (* (seconds timestamp longitude) 100.0) 86400.0) 100.0)))

(println
  (if (= (length args) 1)
    (geotime (string->number (first args)) (realtime))
    (if (= (length args) 2)
      (geotime (string->number (first args)) (string->number (second args)))
      "Usage: geotime <longitude> [<timestamp>]")))
