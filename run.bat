@echo off

set RUST_LOG=pacman=info

pushd assets

..\windows\x64\Debug\PacMan.exe

popd
