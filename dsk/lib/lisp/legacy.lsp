(var eq equal?)
(var atom atom?)
(var add +)
(var sub -)
(var mul *)
(var div /)
(var rem %)
(var exp **)
(var bitand &)
(var bitxor ^)
(var bitor |)

(var car head)
(var cdr tail)

(def (caar x)
  (car (car x)))

(def (cadr x)
  (car (cdr x)))

(def (cdar x)
  (cdr (car x)))

(def (cddr x)
  (cdr (cdr x)))

(var label
  (macro args `(variable ,@args)))

(var lambda
  (macro args `(function ,@args)))

(var progn
  (macro args `(do ,@args)))

(var begin
  (macro args `(do ,@args)))
