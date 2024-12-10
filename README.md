# Multiplayer Testing

This is a basic implementation of multiplayer in Rust for future projects, in this case based on SDL2, but in the future it will be setted to work on Pankarta Software.
---
## How to use it
First you have to get the ipv4 from your device, remember that for this you should use the comand `ifconfig` or `ipconfig` based on your OS.

With that data we can:
1. Use `cargo run` to start the app on 2 different terminal windows.
2. In one run of the command `server` and you will get a message like this: `- ip: 0.0.0.0:12345`.
3. In the other run the command `client` and then write the ipv4 value of your machine (replacing the 0.0.0.0) and then add the `:port_number`
4. Repeat on every instance of the terminal you want.
---


