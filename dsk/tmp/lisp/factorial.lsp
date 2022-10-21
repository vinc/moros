(load "/lib/lisp/core.lsp")

(def (factorial-helper n acc)
  (if (< n 2) acc
    (factorial-helper (- n 1) (* acc n))))

(def (factorial n)
  (factorial-helper n 1))

(println
  (if (nil? args) "Usage: factorial <num>"
    (factorial (string->number (car args)))))
