Things are changing fast. This is the situation on 2019-09-09. LucianoBestia
# mem4_game
Learning Rust Wasm/WebAssembly with Virtual Dom Dodrio and WebSocket communication - part four.
## Documentation
Documentation generated from source code:  
https://lucianobestia.github.io/mem4_game/mem4/index.html  
The workspace mem4_game is made of:  
1. Wasm/WebAssembly  (for browser) frontend - mem4  
2. web server Warp backend - mem4_server  
3. common structures - mem4_common  
## Info and working game
Read the `Last project`:  
https://github.com/LucianoBestia/mem3_game  
You can play the game here (hosted on google cloud platform):  
http://bestia.shorturl.com/mem4  
For exercise I made a Docker image/container for mem4 on the google VM:  
http://bestia.shorturl.com/memdock4  
## Shorturl.com
Google cloud platform does not give any subdomain name for free. Google is also a domain registrar and it looks like they are trying to push me to buy a domain.  
I didn't like to have the raw IP in the url. People don't like numbers like that.  
I created a subdomain on shorturl.com. It is not the perfect solution, but it is free or very cheap.  
## Cargo make
I prepared some flows and tasks for Cargo make for the workspace.  
`cargo make` - lists the possible available/public flows/tasks  
`cargo make dev` - builds the development version and runs the server and the browser  
`cargo make release` - builds the release version and runs the server and the browser  
`cargo make doc` - build the `/target/docs` folder and copy to the `/docs` folder.  

## TODO:
- increase version automatically on build release
- sync data from player1 to others after reconnect.
- different content for English learning: numbers (cardinal, ordinal), food, orientation, alphabet simple spelling, drinks, days/months, questions, colors, transportation, ... 
- on card click only the active player calculates the new state and send the new info to others in a msg  
- fullscreen from http://robnyman.github.io/fullscreen/  
- onfullscreen vdom schedule render  
- iPhone/android webapp manifest file  
- why/how to reset/reload the webapp in "add to homescreen" on iPhone?  

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
2019-07-26 full screen, js snippet, external C, logmod,  
2019-09-08 content filenames in config.json, async fetch game_config.json, separate module for that  
2019-09-09 version 19.9.9 no need for semver if it is not a public library, version in docs  

