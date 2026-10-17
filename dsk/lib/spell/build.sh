set -e
file="scowl.tar.gz"
base="http://downloads.sourceforge.net/wordlist"
curl -sSL "$base/scowl-2020.12.07.tar.gz" -z "$file" -o "$file"
tar xf "$file"
cat scowl-*/final/english-*.{10,20,35,40,50} | \
  iconv -f ISO-8859-1 -t UTF-8 | sort > english.dict
pigz --zlib --best --suffix .z english.dict
