Things are changing fast. This is the situation on 2019-05-21. LucianoBestia
# mem4_game
Learning Rust Wasm/WebAssembly with Virtual Dom Dodrio and WebSocket communication - part four.
Read the `Last project`:  
https://github.com/LucianoBestia/mem3_game  
You can play the game here (hosted on google cloud platform):  
http://34.87.17.103/mem4  
For exercise I made a Docker image/container for mem4 on the google VM:  
http://34.87.17.103/memdock4  
# Idea
Playing the memory game alone is boring.  
Playing it with one friend is better.  
But if both friends just stare in their smartphones, it is still boring.  
What makes memory games (and other board games) entertaining is the company of friends.  
There must be many friends around the table watching one another and stealing moves and laughing at each other.  
Today I assume everybody has a decent smartphone. If all friends open the mem4 game and put their smartphones on the table one near the other, everybody can see them and touch them, this is the closest to a classic board game it gets.  
All the phones will have a 4x4 card grid. If we put 4 smartphones on the table it is now a 8x8 game. That is now much more interesting to play for 4 players.  
It can be played with as many friends as there are: 3,4,5,6,7,8,... More friends - more fun.  
##Typed html
Writing html inside Rust code is much easier with the macro `html!` from the `crate typed-html`  
https://github.com/bodil/typed-html  
It has also a macro `dodrio!` created exclusively for the dodrio vdom.  
Everything is done in compile time, so the runtime is nothing slower.
## TODO:
- better documentation. Do I really have to write very long doc-comments in the code ? It looks terrible. But it is useful when reading the code. Maybe I can hide it in a region block. Dodrio has beautiful docs. How did he do it?  
- cargo release is a subcommand to increment the version and make a git tag

## Changelog
2019-05-24 completed a working version  
2019-06-06 google cloud platform, docker, DockerHub  
2019-06-13 typed html  
2019-06-20 added Triestine, calculate grid size in rust  
  
