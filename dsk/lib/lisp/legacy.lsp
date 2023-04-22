(def (car x)
  (head x))

(def (cdr x)
  (tail x))

(def (caar x)
  (car (car x)))

(def (cadr x)
  (car (cdr x)))

(def (cdar x)
  (cdr (car x)))

(def (cddr x)
  (cdr (cdr x)))

(def (atom x)
  (atom? x))

(def (eq x y)
  (equal? x y))

(var label
  (macro args `(variable ,@args)))

(var lambda
  (macro args `(function ,@args)))

(var progn
  (macro args `(do ,@args)))

(var begin
  (macro args `(do ,@args)))
