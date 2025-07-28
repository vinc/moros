(def (ini/parse path) (do
  (set res (dict))
  (if (file/exists? path) (do
    (set pairs (lines (read path)))
    (set i 0)
    (while (< i (len pairs)) (do
      (set pair (str/split (get pairs i) "="))
      (if (= (len pair) 2)
        (set res (put res (str/trim (first pair)) (str/trim (second pair)))))
      (set i (+ i 1))))))
  res))
