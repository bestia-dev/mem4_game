**mem4_server - html and WebSocket server for the mem4 game**

[comment]: # (lmake_readme version)  
Look also at the workspace readme https://github.com/LucianoBestia/mem4_game  

## mem4_server
Primarily made for learning to code Rust for a http + WebSocket server on the same port  
Using Warp for a simple memory game for kids - mem4.  
On the local public IP address on port 80 listens to http and WebSocket.  
Route for http `/` serves static files from folder `/mem4/`  
Route `/mem4ws/` broadcast all WebSocket msg to all connected clients except sender  

## Google vm
One working server is installed on google vm.  
You can play the game here (hosted on google cloud platform):  
http://bestia.shorturl.com/mem4  



