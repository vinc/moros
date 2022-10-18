(def (eq? x y)
  (eq x y))

(def (atom? x)
  (atom x))

(def (string? x)
  (eq? (type x) "string"))

(def (boolean? x)
  (eq? (type x) "boolean"))

(def (symbol? x)
  (eq? (type x) "symbol"))

(def (number? x)
  (eq? (type x) "number"))

(def (list? x)
  (eq? (type x) "list"))

(def (function? x)
  (eq? (type x) "function"))

(def nil '())

(def (nil? x)
  (eq? x nil))

(def (and x y)
  (if x (if y true) false))

(def (not x)
  (if x false true))

(def (or x y)
  (if x true (if y true) false))

(def (rest x)
  (cdr x))

(def (first x)
  (car x))

(def (second x)
  (first (rest x)))

(def (third x)
  (second (rest x)))

(def (reduce f ls)
  (if (nil? (rest ls)) (first ls)
    (f (first ls) (reduce f (rest ls)))))

(def (string-join ls s)
  (reduce (fn (x y) (string x s y)) ls))

(def (map f ls)
  (if (nil? ls) nil
    (cons
      (f (first ls))
      (map f (rest ls)))))

(def (append x y)
  (if (nil? x) y
    (cons (first x) (append (rest x) y))))

(def (reverse x)
  (if (nil? x) x
    (append (reverse (rest x)) (cons (first x) '()))))

(def (range i n)
  (if (= i n) nil
    (append (list i) (range (+ i 1) n))))

(def (read-line)
  (bytes->string (reverse (rest (reverse (read-file-bytes "/dev/console" 256))))))

(def (read-char)
  (bytes->string (read-file-bytes "/dev/console" 4)))

(def (print exp)
  (do
    (append-file-bytes "/dev/console" (string->bytes (string exp)))
    '()))

(def (println exp)
  (do
    (print exp)
    (print "\n")))

(def (uptime)
  (bytes->number (read-file-bytes "/dev/clk/uptime" 8) "float"))

(def (realtime)
  (bytes->number (read-file-bytes "realtime" 8) "float"))

(def (write-file path str)
  (write-file-bytes path (string->bytes str)))

(def (append-file path str)
  (append-file-bytes path (string->bytes str)))

(def (regex-match? pattern str)
  (not (nil? (regex-find pattern str))))
