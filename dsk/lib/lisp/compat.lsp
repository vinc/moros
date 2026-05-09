(var eq eq?)
(var atom atom?)

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
  (mac args `(var ,@args)))

(var lambda
  (mac args `(fun ,@args)))

(var progn
  (mac args `(do ,@args)))

(var begin
  (mac args `(do ,@args)))
