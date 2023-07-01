(var stdin 0)
(var stdout 1)
(var stderr 2)

# Read

(def (read-binary path)
  "Read binary file"
  (do
    (var size (file.size path))
    (var file (file.open path "r"))
    (var data (file.read file size))
    (file.close file)
    data))

(def (read path)
  "Read text file"
  (binary->string (read-binary path)))

# Write

(def (write-binary path data)
  "Write binary to file"
  (do
    (var file (file.open path "w"))
    (file.write file data)
    (file.close file)))

(def (write path text)
  "Write text to file"
  (write-binary path (string->binary text)))

# Append

(def (append-binary path data)
  "Append binary to file"
  (do
    (var file (file.open path "a"))
    (file.write file data)
    (file.close file)))

(def (append path text)
  "Append text to file"
  (append-binary path (string->binary text)))

# Console

(def (read-line)
  "Read line from the console"
  (string.trim (binary->string (file.read stdin 256))))

(def (read-char)
  "Read char from the console"
  (binary->string (file.read stdin 4)))

(def (p exp)
  "Print expression to the console"
  (do
    (file.write stdout (string->binary (string exp)))
    '()))

(def (print exp)
  "Print expression to the console with a newline"
  (p (string exp "\n")))

# Special

(def (uptime)
  (binary->number (read-binary "/dev/clk/uptime") "float"))

(def (realtime)
  (binary->number (read-binary "/dev/clk/realtime") "float"))
