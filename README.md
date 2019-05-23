Things are changing fast. This is the situation on 2019-05-21. LucianoBestia
# mem4_game
Learning Rust Wasm/WebAssembly with Virtual Dom Dodrio and WebSocket communication - part four.
Read the `Last project`:  
https://github.com/LucianoBestia/mem3_game  
# Idea
Playing the memory game alone is boring.  
Playing it with one friend is better.  
But if both friends just stare in their smartphones, it is still boring.  
What makes memory games (and other board games) entertaining is the company of friends.  
There must be many friends around the table watching one another and stealing moves and laughing at each other.  
Today I assume everybody has a decent smartphone. If all friends open the mem4 game and put their smartphones on the table one near the other, everybody can see them and touch them, this is the closest to a classic board game it gets.  
All the phones will have a 4x4 card grid. If we put 4 smartphones on the table it is now a 8x8 game. That is now much more interesting to play for 4 players.  
It can be played with as many friends as there are: 3,4,5,6,7,8,... More friends - more fun.  
## TODO:
In mem4 I plan to:
- Invite  players
- a number of players Accept play.
- the inviter stops the invite when all accepted
- now we have a sequence of players: player1, player2, player3,...
- every player has a 4x4 grid on the smartphone 
- indexing: player1 from 1 to 16, player2 from 17 to 32, player3 from 33 to 49
- the inviter calculates 8 x playersCount random numbers r.g. 8x4=32 for 4 players
- the random numbers can duplicate, so we don't get out of options if there are a great number of players.
- we need pairs, so we copy the same random numbers once, then shuffle
- now we can render them 
- all the cards on all the phones are active to flip over





- better documentation. Do I really have to write very long doc-comments in the code ? It looks terrible. But it is useful when reading the code. Maybe I can hide it in a region block. Dodrio has beautiful docs. How did he do it?  


