(def (dict/keys d)
  "Returns the keys of a dict"
  (map first (dict/pairs d)))

(def (dict/values d)
  "Returns the values of a dict"
  (map last (dict/pairs d)))
