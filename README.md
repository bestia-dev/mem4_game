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
http://bestia.dev/mem4  
For exercise I made a Docker image/container for mem4 on the google VM:  
http://bestia.dev/memdock4  
## bestia.dev
Google cloud platform does not give any subdomain name for free.  
Google is also a domain registrar and it looks like they are trying to push me to buy their domain.   
I had to buy my own domain name bestia.dev from porkbun.com for 12$ per year.
## Cargo make
I prepared some flows and tasks for Cargo make for the workspace.  
`cargo make` - lists the possible available/public flows/tasks  
`cargo make dev` - builds the development version and runs the server and the browser  
`cargo make release` - builds the release version and runs the server and the browser  
`cargo make doc` - build the `/target/docs` folder and copy to the `/docs` folder.  

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
2019-09-10 increase version automatically on build  
2019-09-13 end development of mem4, start mem5

