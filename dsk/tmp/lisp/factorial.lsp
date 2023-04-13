(load "/lib/lisp/core.lsp")

(def (factorial-helper n acc)
  (if (< n 2) acc
    (factorial-helper (- n 1) (* acc n))))

(def (factorial n)
  (factorial-helper n 1))

(print
  (if (nil? args) "Usage: factorial <num>"
    (factorial (str->num (head args)))))
