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

(def-mac (or @xs)
  (if (nil? xs) false
    (if (nil? (tail xs)) (head xs)
      `(if ,(head xs) ,(head xs) (or ,@(tail xs))))))

(def-mac (and @xs)
  (if (nil? xs) true
    (if (nil? (tail xs)) (head xs)
      `(if ,(head xs) (and ,@(tail xs)) false))))

(def (empty? x)
  (= (len x) 0))

(def-mac (set k v)
  `(if (var? ,k)
     (mut ,k ,v)
     (var ,k ,v)))

(def-mac (let params values body)
  `((fun ,params ,body) ,@values))

(def-mac (push! l x)
  `(mut ,l (push ,l ,x)))

(def-mac (put! d k v)
  `(mut ,d (put ,d ,k ,v)))

(def (reduce f ls)
  "Reduces the elements of the list with the function"
  (if (empty? (tail ls)) (head ls)
    (f (head ls) (reduce f (tail ls)))))

(def (map f ls)
  "Applies the function to the elements of the list"
  (if (empty? ls) nil
    (cons
      (f (head ls))
      (map f (tail ls)))))

(def (filter f ls)
  "Filters the elements of the list with the function"
  (if (empty? ls) nil
    (if (f (head ls))
      (cons (head ls) (filter f (tail ls)))
      (filter f (tail ls)))))

(def (reject f ls)
  "Rejects the elements of the list with the function"
  (if (empty? ls) nil
    (if (not (f (head ls)))
      (cons (head ls) (reject f (tail ls)))
      (reject f (tail ls)))))

(def (intersection a b)
  "Returns the elements found in both lists"
  (filter (fun (x) (contains? b x)) a))

(def (reverse ls)
  "Reverses the list"
  (if (nil? ls) ls
    (concat (reverse (tail ls)) (cons (head ls) '()))))

(def (range start stop)
  "Returns a list of numbers from start to stop excluded"
  (if (= start stop) nil
    (concat (list start) (range (+ start 1) stop))))

(def (min ls)
  "Returns the minimum element of the list"
  (head (sort ls)))

(def (max ls)
  "Returns the maximum element of the list"
  (head (reverse (sort ls))))

(def (abs x)
  "Returns the absolute value of the number"
  (if (> x 0) x (- x)))

(def (mod a b)
  "Returns the remainder of the division"
  (% (+ (% a b) b) b))

(def (string/join ls s)
  "Joins the elements of the list with the string"
  (if (empty? ls) "" (reduce (fun (x y) (string x s y)) ls)))

(def (regex/match? r s)
  "Returns true if the string match the pattern"
  (not (nil? (regex/find r s))))

(def (lines text)
  "Splits the text into a list of lines"
  (string/split (string/trim text) "\n"))

(def (words text)
  "Splits the text into a list of words"
  (string/split text " "))

(def (chars text)
  "Splits the text into a list of chars"
  (string/split text ""))

(def (push ls x)
  "Pushes the element to the end of the list"
  (put ls (len ls) x))

(def (first ls)
  "Returns the first element of the list"
  (get ls 0))

(def (second ls)
  "Returns the second element of the list"
  (get ls 1))

(def (third ls)
  "Returns the third element of the list"
  (get ls 2))

(def (last ls)
  "Returns the last element of the list"
  (get ls
    (if (= (length ls) 0) 0 (- (length ls) 1))))

(def (shell->string cmd)
  "Returns the output of the command"
  (string/trim (binary->string (shell->binary cmd))))

# Short aliases

(var str/join string/join)
(var bool? boolean?)
(var str? string?)
(var sym? symbol?)
(var num? number?)
(var fun? function?)
(var mac? macro?)
(var rev reverse)
(var sh->str shell->string)

(load "/lib/lisp/dict.lsp")
(load "/lib/lisp/file.lsp")
(load "/lib/lisp/math.lsp")
