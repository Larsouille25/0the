@echo off

cls

echo Compiling the program ...

g++ src/main.cpp include/lwlogger/lwlogger_v1.0.0-rc.dll -o build/Othebot

echo Program compiled !

echo Run the program ...
echo:

cd build
Othebot.exe
cd ..