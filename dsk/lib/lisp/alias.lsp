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

(var eq?
  (macro args `(equal? ,@args)))

(var rest
  (macro args `(tail ,@args)))

(var help
  (macro args `(doc ,@args)))
