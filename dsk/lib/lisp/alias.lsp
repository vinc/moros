(variable var
  (macro args `(variable ,@args)))

(var mac
  (macro args `(macro ,@args)))

(var fun
  (macro args `(function ,@args)))

(var def
  (macro args `(define ,@args)))

(var def-mac
  (macro args `(define-macro ,@args)))

(var def-fun
  (macro args `(define-function ,@args)))

(def (car lst)
  (head lst))

(def (cdr lst)
  (tail lst))

(def (atom x)
  (atom? x))

(def (eq x y)
  (equal? x y))

(def (eq? x y)
  (equal? x y))

(var label
  (macro args `(variable ,@args)))

(var lambda
  (macro args `(function ,@args)))

(var progn
  (macro args `(do ,@args)))

(var begin
  (macro args `(do ,@args)))
