(load "/lib/lisp/core.lsp")

(def (ansi-color x y)
  (str "\e[" x ";" y "m"))

(def (fg c)
  (ansi-color c 40))

(def (bg c)
  (ansi-color 30 c))

(def (color f c)
  (str " " (f c) (if (< c 100) " " "") c (ansi-color 0 0)))

(def (colors fs i j)
  (str/join (map (fun (c) (color fs c)) (range i j)) ""))

(print (colors fg 30 38))
(print (colors fg 90 98))
(print (colors bg 40 48))
(print (colors bg 100 108))
