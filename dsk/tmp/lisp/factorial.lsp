(load "/ini/lisp/core.lsp")

(defn fact-acc (n acc)
  (cond
    ((< n 2) acc)
    (true (fact-acc (- n 1) (* acc n)))))

(defn fact (n)
  (fact-acc n 1))

(println
  (fact
    (cond
      ((null? args) 10)
      (true (parse (car args))))))
