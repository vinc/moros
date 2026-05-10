(load "/lib/lisp/core.lsp")

(def (print-doc f) (do
  (set s (second (parse (str (eval f)))))
  (set d (doc (eval f)))
  (print (str
    "("
    (if (fun? (eval f)) "\e[96m" "\e[92m") f "\e[0m" # name
    (if (nil? s) ""
      (str " " (if (list? s) (str/join s " ") s))) # args
    ")"
    "\e[90m" (if (empty? d) "" " # ") d "\e[0m")))) # desc

(set fs
  (filter
    (fun (f) (or (fun? (eval f)) (mac? (eval f))))
    (env)))

(map print-doc fs)
