@echo off

set base=%~dp0

call "C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Auxiliary\Build\vcvarsall.bat" x64

cd %base%

pushd .\windows

msbuild

popd
