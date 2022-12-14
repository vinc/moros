(load "/lib/lisp/alias.lsp")

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

(define (macro? x)
  (eq? (type x) "macro"))

(define nil '())

(define (nil? x)
  (eq? x nil))

(define (not x)
  (if x false true))

(define-macro (or x y)
  `(if ,x true (if ,y true false)))

(define-macro (and x y)
  `(if ,x (if ,y true false) false))

(define-macro (let params values body)
  `((function ,params ,body) ,@values))

(define (reduce f ls)
  (if (nil? (tail ls)) (head ls)
    (f (head ls) (reduce f (tail ls)))))

(define (map f ls)
  (if (nil? ls) nil
    (cons
      (f (head ls))
      (map f (tail ls)))))

(define (filter f ls)
  (if (nil? ls) nil
    (if (f (head ls))
      (cons (head ls) (filter f (tail ls)))
      (filter f (tail ls)))))

(define (intersection a b)
  (filter (function (x) (contains? b x)) a))

(define (reverse x)
  (if (nil? x) x
    (append (reverse (tail x)) (cons (head x) '()))))

(define (range i n)
  (if (= i n) nil
    (append (list i) (range (+ i 1) n))))

(define (min lst)
  (head (sort lst)))

(define (max lst)
  (head (reverse (sort lst))))

(define (abs x)
  (if (> x 0) x (- x)))

(define (string-join ls s)
  (reduce (function (x y) (string x s y)) ls))

(define (read-line)
  (bytes->string (reverse (tail (reverse (read-file-bytes "/dev/console" 256))))))

(define (read-char)
  (bytes->string (read-file-bytes "/dev/console" 4)))

(define (print exp)
  (do
    (append-file-bytes "/dev/console" (string->bytes (string exp)))
    '()))

(define (println exp)
  (print (string exp "\n")))

(define (uptime)
  (bytes->number (read-file-bytes "/dev/clk/uptime" 8) "float"))

(define (realtime)
  (bytes->number (read-file-bytes "/dev/clk/realtime" 8) "float"))

(define (write-file path str)
  (write-file-bytes path (string->bytes str)))

(define (append-file path str)
  (append-file-bytes path (string->bytes str)))

(define (regex-match? pattern str)
  (not (nil? (regex-find pattern str))))

(define (lines contents)
  (split (trim contents) "\n"))

(define (words contents)
  (split contents " "))

(define (chars contents)
  (split contents ""))

(define (first lst)
  (nth lst 0))

(define (second lst)
  (nth lst 1))

(define (third lst)
  (nth lst 2))

(define (last lst)
  (nth lst
    (if (= (length lst) 0) 0 (- (length lst) 1))))

(define (caar x)
  (car (car x)))

(define (cadr x)
  (car (cdr x)))

(define (cdar x)
  (cdr (car x)))

(define (cddr x)
  (cdr (cdr x)))

(define rest cdr)
(define len length)
(define rev reverse)
(define uniq unique)
