set -e

wget -c https://github.com/copy/v86/releases/download/latest/libv86.js
wget -c https://github.com/copy/v86/releases/download/latest/v86.wasm
wget -c https://github.com/copy/v86/raw/refs/heads/master/bios/seabios.bin
wget -c https://github.com/copy/v86/raw/refs/heads/master/bios/vgabios.bin

# Fix build issue
perl -pi -e "s/P\.clearRect\(0,xa,N,U\);let Ia,Pa,Qa;/P.clearRect(0,xa,N,U);L=void 0;let Ia,Pa,Qa;/" libv86.js
