(load "/lib/lisp/alias.lsp")

(def (string? x)
  (equal? (type x) "string"))

(def (boolean? x)
  (equal? (type x) "boolean"))

(def (symbol? x)
  (equal? (type x) "symbol"))

(def (number? x)
  (equal? (type x) "number"))

(def (list? x)
  (equal? (type x) "list"))

(def (function? x)
  (equal? (type x) "function"))

(def (macro? x)
  (equal? (type x) "macro"))

(var nil '())

(def (nil? x)
  (equal? x nil))

(def (not x)
  (if x false true))

(def-mac (or x y)
  `(if ,x true (if ,y true false)))

(def-mac (and x y)
  `(if ,x (if ,y true false) false))

(def-mac (let params values body)
  `((fun ,params ,body) ,@values))

(def (reduce f ls)
  (if (nil? (tail ls)) (head ls)
    (f (head ls) (reduce f (tail ls)))))

(def (map f ls)
  (if (nil? ls) nil
    (cons
      (f (head ls))
      (map f (tail ls)))))

(def (filter f ls)
  (if (nil? ls) nil
    (if (f (head ls))
      (cons (head ls) (filter f (tail ls)))
      (filter f (tail ls)))))

(def (intersection a b)
  (filter (fun (x) (contains? b x)) a))

(def (reverse x)
  (if (nil? x) x
    (append (reverse (tail x)) (cons (head x) '()))))

(def (range i n)
  (if (= i n) nil
    (append (list i) (range (+ i 1) n))))

(def (min lst)
  (head (sort lst)))

(def (max lst)
  (head (reverse (sort lst))))

(def (abs x)
  (if (> x 0) x (- x)))

(def (join-string ls s)
  (reduce (fun (x y) (string x s y)) ls))

(def (read-line)
  (bytes->string (reverse (tail (reverse (read-file-bytes "/dev/console" 256))))))

(def (read-char)
  (bytes->string (read-file-bytes "/dev/console" 4)))

(def (p exp)
  (do
    (append-file-bytes "/dev/console" (string->bytes (string exp)))
    '()))

(def (print exp)
  (p (string exp "\n")))

(def (uptime)
  (bytes->number (read-file-bytes "/dev/clk/uptime" 8) "float"))

(def (realtime)
  (bytes->number (read-file-bytes "/dev/clk/realtime" 8) "float"))

(def (write-file path s)
  (write-file-bytes path (string->bytes s)))

(def (append-file path s)
  (append-file-bytes path (string->bytes s)))

(def (regex-match? pattern s)
  (not (nil? (regex-find pattern str))))

(def (lines contents)
  (split (trim contents) "\n"))

(def (words contents)
  (split contents " "))

(def (chars contents)
  (split contents ""))

(def (first lst)
  (nth lst 0))

(def (second lst)
  (nth lst 1))

(def (third lst)
  (nth lst 2))

(def (last lst)
  (nth lst
    (if (= (length lst) 0) 0 (- (length lst) 1))))

(def (caar x)
  (car (car x)))

(def (cadr x)
  (car (cdr x)))

(def (cdar x)
  (cdr (car x)))

(def (cddr x)
  (cdr (cdr x)))

(var str string)
(var num-type number-type)
(var join-str join-string)

(var str->num string->number)
(var str->bin string->bytes)
(var num->bin number->bytes)
(var bin->str bytes->string)
(var bin->num bytes->number)

(var bool? boolean?)
(var str? string?)
(var sym? symbol?)
(var num? number?)

(var fun? function?)
(var mac? macro?)

(var rest cdr)
(var len length)
(var rev reverse)
(var uniq unique)
