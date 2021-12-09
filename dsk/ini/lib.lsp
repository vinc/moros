(defn string? (x)
  (eq (type x) "string"))

(defn boolean? (x)
  (eq (type x) "boolean"))

(defn symbol? (x)
  (eq (type x) "symbol"))

(defn number? (x)
  (eq (type x) "number"))

(defn list? (x)
  (eq (type x) "list"))

(defn function? (x)
  (eq (type x) "function"))

(defn lambda? (x)
  (eq (type x) "lambda"))
