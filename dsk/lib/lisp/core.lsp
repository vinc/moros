(load "/lib/lisp/alias.lsp")

(def (string? x)
  (equal? (type x) "string"))

(def (boolean? x)
  (equal? (type x) "boolean"))

(def (symbol? x)
  (equal? (type x) "symbol"))

(def (number? x)
  (equal? (type x) "number"))

(def (list? x)
  (equal? (type x) "list"))

(def (function? x)
  (equal? (type x) "function"))

(def (macro? x)
  (equal? (type x) "macro"))

(var nil '())

(def (nil? x)
  (equal? x nil))

(def (not x)
  (if x false true))

(def-mac (or x y)
  `(if ,x true (if ,y true false)))

(def-mac (and x y)
  `(if ,x (if ,y true false) false))

(def-mac (let params values body)
  `((fun ,params ,body) ,@values))

(def (reduce f ls)
  "Reduce the elements of the list with the function"
  (if (nil? (tail ls)) (head ls)
    (f (head ls) (reduce f (tail ls)))))

(def (map f ls)
  "Apply the function to the elements of the list"
  (if (nil? ls) nil
    (cons
      (f (head ls))
      (map f (tail ls)))))

(def (filter f ls)
  "Filter the elements of the list with the function"
  (if (nil? ls) nil
    (if (f (head ls))
      (cons (head ls) (filter f (tail ls)))
      (filter f (tail ls)))))

(def (intersection a b)
  "Return elements found in both lists"
  (filter (fun (x) (contains? b x)) a))

(def (reverse x)
  "Reverse list"
  (if (nil? x) x
    (concat (reverse (tail x)) (cons (head x) '()))))

(def (range start stop)
  "Return a list of integers from start to stop excluded"
  (if (= start stop) nil
    (concat (list start) (range (+ start 1) stop))))

(def (min lst)
  "Return the minimum element of the list"
  (head (sort lst)))

(def (max lst)
  "Return the maximum element of the list"
  (head (reverse (sort lst))))

(def (abs x)
  (if (> x 0) x (- x)))

(def (mod a b)
  (rem (+ (rem a b) b) b))

(def (string.join ls s)
  "Join the elements of the list with the string"
  (reduce (fun (x y) (string x s y)) ls))

(def (regex.match? pattern s)
  (not (nil? (regex.find pattern str))))

(def (lines text)
  "Split text into a list of lines"
  (string.split (string.trim text) "\n"))

(def (words text)
  "Split text into a list of words"
  (string.split text " "))

(def (chars text)
  "Split text into a list of chars"
  (string.split text ""))

(def (first lst)
  (nth lst 0))

(def (second lst)
  (nth lst 1))

(def (third lst)
  (nth lst 2))

(def (last lst)
  (nth lst
    (if (= (length lst) 0) 0 (- (length lst) 1))))

# Short aliases

(var % rem)
(var str string)
(var str.split string.split)
(var str.join string.join)
(var str.trim string.trim)
(var num.type number.type)
(var str->num string->number)
(var str->bin string->binary)
(var num->bin number->binary)
(var bin->str binary->string)
(var bin->num binary->number)
(var bool? boolean?)
(var str? string?)
(var sym? symbol?)
(var num? number?)
(var fun? function?)
(var mac? macro?)
(var len length)
(var rev reverse)
(var uniq unique)

(load "/lib/lisp/file.lsp")
