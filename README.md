# eletechsup R4D3B16 board controller
## Introduction
A simple lightweight tool to control relay board via Modbus (RS-485).  
Works on Windows and Linux  
Developed with:  
Rust + FLTK + tokio-modbus  
## How to build program
```
cargo build --release
```  
## How to build program for older Linux distributions
To build this program for older Linux (with older glibc version) tou can use cross-rs tool.  
```
cargo install cross
```  
After it you should build docker image
```
cd myimage
docker build -t myimage/builder:0.1.0 .
cd ..
```  
And then run cross
```
cross build --target=x86_64-unknown-linux-gnu --release
```  
## Board
![Alt text](img/board.jpg?raw=true "Board")  
## GUI
![Alt text](img/window.png?raw=true "GUI")  
