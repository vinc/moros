(var =
  (mac args `(eq? ,@args)))

(var help
  (mac args `(doc ,@args)))

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
