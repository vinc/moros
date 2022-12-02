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

(define (caar x)
  (car (car x)))

(define (cadr x)
  (car (cdr x)))

(define (cdar x)
  (cdr (car x)))

(define (cddr x)
  (cdr (cdr x)))

(define (rest x)
  (cdr x))

(define (first x)
  (car x))

(define (second x)
  (first (rest x)))

(define (third x)
  (second (rest x)))

(define (reduce f ls)
  (if (nil? (rest ls)) (first ls)
    (f (first ls) (reduce f (rest ls)))))

(define (map f ls)
  (if (nil? ls) nil
    (cons
      (f (first ls))
      (map f (rest ls)))))

(define (reverse x)
  (if (nil? x) x
    (append (reverse (rest x)) (cons (first x) '()))))

(define (range i n)
  (if (= i n) nil
    (append (list i) (range (+ i 1) n))))

(define (string-join ls s)
  (reduce (function (x y) (string x s y)) ls))

(define (read-line)
  (bytes->string (reverse (rest (reverse (read-file-bytes "/dev/console" 256))))))

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
