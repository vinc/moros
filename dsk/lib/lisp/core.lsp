(define (eq? x y)
  (eq x y))

(define (atom? x)
  (atom x))

(define (string? x)
  (eq? (type x) "string"))

(define (boolean? x)
  (eq? (type x) "boolean"))

(define (symbol? x)
  (eq? (type x) "symbol"))

(define (number? x)
  (eq? (type x) "number"))

(define (list? x)
  (eq? (type x) "list"))

(define (function? x)
  (eq? (type x) "function"))

(define nil '())

(define (nil? x)
  (eq? x nil))

(define (and x y)
  (cond
    (x (cond (y true) (true false)))
    (true false)))

(define (not x)
  (cond (x false) (true true)))

(define (or x y)
  (cond (x true) (y true) (true false)))

(define (rest x)
  (cdr x))

(define (first x)
  (car x))

(define (second x)
  (first (rest x)))

(define (third x)
  (second (rest x)))

(define (reduce f ls)
  (cond
    ((nil? (rest ls)) (first ls))
    (true (f (first ls) (reduce f (rest ls))))))

(define (string-join ls s)
  (reduce (fn (x y) (string x s y)) ls))

(define (map f ls)
  (cond
    ((nil? ls) nil)
    (true (cons
      (f (first ls))
      (map f (rest ls))))))

(define (append x y)
  (cond
    ((nil? x) y)
    (true (cons (first x) (append (rest x) y)))))

(define (reverse x)
  (cond
    ((nil? x) x)
    (true (append (reverse (rest x)) (cons (first x) '())))))

(define (range i n)
  (cond
    ((= i n) nil)
    (true (append (list i) (range (+ i 1) n)))))

(define (read-line)
  (bytes->string (reverse (rest (reverse (read-file-bytes "/dev/console" 256))))))

(define (read-char)
  (bytes->string (read-file-bytes "/dev/console" 4)))

(define (print exp)
  (do
    (append-file-bytes "/dev/console" (string->bytes (string exp)))
    '()))

(define (println exp)
  (do
    (print exp)
    (print "\n")))

(define (uptime)
  (bytes->number (read-file-bytes "/dev/clk/uptime" 8)))

(define (realtime)
  (bytes->number (read-file-bytes "realtime" 8)))

(define (write-file path str)
  (write-file-bytes path (string->bytes str)))

(define (append-file path str)
  (append-file-bytes path (string->bytes str)))

(define (regex-match? pattern str)
  (not (nil? (regex-find pattern str))))
