(load "/lib/lisp/core.lsp")

(define esc (bytes->string '(27)))

(define (ansi-color x y)
  (string esc "[" x ";" y "m"))

(define (fg c)
  (ansi-color c 40))

(define (bg c)
  (ansi-color 30 c))

(define (color f c)
  (string " " (f c) (if (< c 100) " " "") c (ansi-color 0 0)))

(define (colors fs i j)
  (string-join (map (function (c) (color fs c)) (range i j)) ""))

(println (colors fg 30 38))
(println (colors fg 90 98))
(println (colors bg 40 48))
(println (colors bg 100 108))
