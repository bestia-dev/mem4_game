Things are changing fast. This is the situation on 2019-06-25. LucianoBestia
# mem4_game
Learning Rust Wasm/WebAssembly with Virtual Dom Dodrio and WebSocket communication - part four.
## Documentation
Documentation generated from source code:  
https://lucianobestia.github.io/mem4_game/mem4/index.html
## Info and working game
Read the `Last project`:  
https://github.com/LucianoBestia/mem3_game  
You can play the game here (hosted on google cloud platform):  
http://bestia.shorturl.com/mem4  
For exercise I made a Docker image/container for mem4 on the google VM:  
http://bestia.shorturl.com/memdock4  
# Idea
Playing the memory game alone is boring.  
Playing it with one friend is better.  
But if both friends just stare in their smartphones, it is still boring.  
What makes memory games (and other board games) entertaining is the company of friends.  
There must be many friends around the table watching one another and stealing moves and laughing at each other.  
Today I assume everybody has a decent smartphone. If all friends open the mem4 game and put their smartphones on the table one near the other, everybody can see them and touch them, this is the closest to a classic board game it gets.  
All the phones will have a 4x4 card grid. If we put 4 smartphones on the table it is now a 8x8 game. That is now much more interesting to play for 4 players.  
It can be played with as many friends as there are: 3,4,5,6,7,8,... More friends - more fun.  
## Typed html
Writing html inside Rust code is much easier with the macro `html!` from the `crate typed-html`  
https://github.com/bodil/typed-html  
It has also a macro `dodrio!` created exclusively for the dodrio vdom.  
Everything is done in compile time, so the runtime is nothing slower.
## WS reconnect
It looks that plain web sockets have often connection problems and they disconnect here and there. Creating a good reconnect it pretty challenging. 
## Cargo make
I prepared some flows and tasks for Cargo make.  
`cargo make` - lists the possible available/public flows/tasks  
`cargo make dev` - builds the development version and runs the server and the browser  
`cargo make release` - builds the release version and runs the server and the browser  
`cargo make doc` - build the `/target/docs` folder. Copying to the `/docs` folder must be manually performed for now.  
## Shorturl.com
Google cloud platform does not give any subdomain name for free. Google is also a domain registrar and it looks like they are trying to push me to buy a domain.  
I didn't like to have the raw IP in the url. People don't like numbers like that.  
I created a subdomain on shorturl.com. It is not the perfect solution, but it is free or very cheap.  
## TODO:
- sync data from player1 to others after reconnect.
- sometimes the grid is bigger after every click, but sometimes not
- put the version number in the application, increase version on build release
- put filenames is game_config.json, to have more freedom with filenames
- different content for English learning: numbers (cardinal, ordinal), food, orientation, alphabet simple spelling, drinks, days/months, questions, colors, transportation, ... 
- on card click only the active player calculates the new state and send the new info to others in a msg
- fullscreen from http://robnyman.github.io/fullscreen/
- use fetch module instead of websockets for json
## Changelog
2019-05-24 completed a working version  
2019-06-06 google cloud platform, docker, DockerHub  
2019-06-13 typed html  
2019-06-20 added Triestine, calculate grid size in rust  
2019-06-25 integer arithmetic unwrap!(x.checked_add(y)), checked_sub, checked_mul, checked_div  
- floating f64 cast to usize and vice versa with x.approx_as::<f64>() from crate conv  
- manual reconnect for ws disconnect  
2019-06-26 3x3 or 3x2 grids. GameConfig.  
2019-07-03 game flow in modules  
2019-07-05 refactoring, refactoring,...  
2019-07-09 request_fullscreen button  

  
