set -o xtrace
ncat -klv 1025 -c "ncat -v --ssl $1 $2"
