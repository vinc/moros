(variable var
  (macro args `(variable ,@args)))

(var var?
  (macro args `(variable? ,@args)))

(var mut
  (macro args `(mutate ,@args)))

(var mac
  (macro args `(macro ,@args)))

(var fun
  (macro args `(function ,@args)))

(var def
  (macro args `(define ,@args)))

(var def-mac
  (macro args `(define-macro ,@args)))

(var def-fun
  (macro args `(define-function ,@args)))

(var eq?
  (macro args `(equal? ,@args)))

(var rest
  (macro args `(tail ,@args)))

(var help
  (macro args `(doc ,@args)))

# Primitive aliases

(var >= gte?)
(var <= lte?)
(var > gt?)
(var < lt?)
(var + add)
(var - sub)
(var * mul)
(var / div)
(var ** exp)
(var % rem)
(var & bit/and)
(var ^ bit/xor)
(var | bit/or)
(var << bit/shl)
(var >> bit/shr)
(var sh->bin shell->binary)
(var sh shell)
(var $ shell)
(var str string)
(var str/split string/split)
(var str/trim string/trim)
(var num/type number/type)
(var num/int number/int)
(var str->num string->number)
(var str->bin string->binary)
(var num->bin number->binary)
(var num->str number->string)
(var bin->str binary->string)
(var bin->num binary->number)
(var len length)
(var uniq unique)
