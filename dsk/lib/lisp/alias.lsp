(define def
  (macro args `(define ,@args)))

(define mac
  (macro args `(macro ,@args)))

(define fun
  (macro args `(function ,@args)))

(define def-mac
  (macro args `(define-macro ,@args)))

(define def-fun
  (macro args `(define-function ,@args)))

(define (car lst)
  (head lst))

(define (cdr lst)
  (tail lst))

(define label
  (macro args `(define ,@args)))

(define lambda
  (macro args `(function ,@args)))

(define progn
  (macro args `(do ,@args)))

(define begin
  (macro args `(do ,@args)))
