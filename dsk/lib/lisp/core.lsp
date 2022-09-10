(defn eq? (x y)
  (eq x y))

(defn atom? (x)
  (atom x))

(defn string? (x)
  (eq? (type x) "string"))

(defn boolean? (x)
  (eq? (type x) "boolean"))

(defn symbol? (x)
  (eq? (type x) "symbol"))

(defn number? (x)
  (eq? (type x) "number"))

(defn list? (x)
  (eq? (type x) "list"))

(defn function? (x)
  (eq? (type x) "function"))

(defn lambda? (x)
  (eq? (type x) "lambda"))

(def null '())

(defn null? (x)
  (eq? x null))

(defn and (x y)
  (cond
    (x (cond (y true) (true false)))
    (true false)))

(defn not (x)
  (cond (x false) (true true)))

(defn or (x y)
  (cond (x true) (y true) (true false)))

(defn rest (x)
  (cdr x))

(defn first (x)
  (car x))

(defn second (x)
  (first (rest x)))

(defn third (x)
  (second (rest x)))

(defn reduce (f ls)
  (cond
    ((null? (rest ls)) (first ls))
    (true (f (first ls) (reduce f (rest ls))))))

(defn string-join (ls s)
  (reduce (fn (x y) (string x s y)) ls))

(defn map (f ls)
  (cond
    ((null? ls) null)
    (true (cons
      (f (first ls))
      (map f (rest ls))))))

(defn append (x y)
  (cond
    ((null? x) y)
    (true (cons (first x) (append (rest x) y)))))

(defn reverse (x)
  (cond
    ((null? x) x)
    (true (append (reverse (rest x)) (cons (first x) '())))))

(defn range (i n)
  (cond
    ((= i n) null)
    (true (append (list i) (range (+ i 1) n)))))

(defn read-line ()
  (decode-string (reverse (rest (reverse (read-file-bytes "/dev/console" 256))))))

(defn read-char ()
  (decode-string (read-file-bytes "/dev/console" 4)))

(defn print (exp)
  (do (append-file-bytes "/dev/console" (encode-string (string exp))) '()))

(defn println (exp)
  (do (print exp) (print "\n")))

(def pr print)
(def prn println)

(defn uptime ()
  (decode-number (read-file-bytes "/dev/clk/uptime" 8)))

(defn realtime ()
  (decode-number (read-file-bytes "realtime" 8)))

(defn write-file (path str)
  (write-file-bytes path (encode-string str)))

(defn append-file (path str)
  (append-file-bytes path (encode-string str)))

(defn regex-match? (pattern str)
  (not (null? (regex-find pattern str))))
