(def (dict/keys d)
  "Returns the keys of a dict"
  (map (fun (args) (first args)) d))

(def (dict/values d)
  "Returns the values of a dict"
  (map (fun (args) (last args)) d))
