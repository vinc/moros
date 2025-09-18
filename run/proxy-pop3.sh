set -o xtrace
ncat -klv 10110 -c "ncat -v --ssl $1 $2"
