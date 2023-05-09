set -e

ln -fs ../doc/*.png .

echo "# MOROS" > ../doc/test.md

for md in ../doc/*.md; do
  title="$(head -n 1 $md | sed "s/^#* //")"
  html="$(basename ${md%.*}.html)"
  echo "$md => $html"
  cat << EOF > $html
<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <title>$title</title>
    <link rel="stylesheet" type="text/css" href="moros.css">
  </head>
  <body>
EOF
  redcarpet --parse fenced-code-blocks ../doc/$md | sed "s/.md/.html/g" | sed "s/^</    </" | sed "s/    <\/code/<\/code/" >> $html
  cat << EOF >> $html
  </body>
</html>
EOF
done

rm ../doc/test.md
