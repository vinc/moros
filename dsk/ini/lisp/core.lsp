(defn eq? (a b)
  (eq a b))

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

(defn rest (a)
  (cdr a))

(defn first (a)
  (car a))

(defn second (a)
  (first (rest a)))

(defn third (a)
  (second (rest a)))

(defn append (a b)
  (cond
    ((null? a) b)
    (true (cons (first a) (append (rest a) b)))))

(defn reverse (a)
  (cond
    ((null? a) a)
    (true (append (reverse (rest a)) (cons (first a) '())))))

(defn read-line ()
  (decode-string (reverse (rest (reverse (read-file-bytes "/dev/console" 256))))))

(defn read-char ()
  (decode-string (read-file-bytes "/dev/console" 4)))

(defn print (exp)
  (do (append-file-bytes "/dev/console" (encode-string (string exp))) '()))

(defn println (exp)
  (do (print exp) (print "\n")))

(defn uptime ()
  (decode-number (read-file-bytes "/dev/clk/uptime" 8)))

(defn write-file (path str)
  (write-file-bytes path (encode-string str)))

(defn append-file (path str)
  (append-file-bytes path (encode-string str)))
