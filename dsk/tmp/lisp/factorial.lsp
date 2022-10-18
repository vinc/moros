(load "/lib/lisp/core.lsp")

(def (factorial-helper n acc)
  (cond
    ((< n 2) acc)
    (true (factorial-helper (- n 1) (* acc n)))))

(def (factorial n)
  (factorial-helper n 1))

(println
  (cond
    ((nil? args) "Usage: factorial <num>")
    (true (factorial (string->number (car args))))))
