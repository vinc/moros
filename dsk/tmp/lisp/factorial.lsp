(load "/ini/lisp/core.lsp")

(defn fact-acc (n acc)
  (cond
    ((< n 2) acc)
    (true (fact-acc (- n 1) (* acc n)))))

(defn fact (n)
  (fact-acc n 1))

(println
  (cond
    ((null? args) "Usage: factorial <num>")
    (true (fact (parse (car args))))))
