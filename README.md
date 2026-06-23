# local char

local char is a cli and a web program that runs ai locally that can act like the character you want, although so far the cli part is fully finished not the website and the characters are mostly mad for 3-5 b models not very strong im going to add different category for the range strength of each model so it can act better like your desired character

## features
- locally run ai with llama.cpp
- has a cli
- currently have 15 characters
- chat history
- made for weaker models (for now)
- chat history
- great tui
- really basic web server

## setup

This project expects **llama.cpp running**.

You need to start `llama.cpp` in server mode so it exposes an API endpoint:

Example:

```bash
git clone https://github.com/ggml-org/llama.cpp.git
cd llama.cpp
./build/bin/llama-server -m PATH-TO-YOUR-MODEL.gguf
```

## install

make sure llama.cpp server is running

```bash
git clone https://github.com/KishaWeb/local-char.git
cd local-char
cargo install --path .
local-char
```

## usage
```bash
local-char tui (runs tui)
local-char web (starts the web server)
```

if your running it in cargo use it like this:

```bash
cargo run -- tui (runs tui)
cargo run -- web (starts web server)
```