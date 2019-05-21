//! rules and descriptions
//! is a static content. great for cache.

use crate::text_with_br_newline;
use dodrio::builder::*;
use dodrio::bumpalo::{self, Bump};
use dodrio::{Node, Render};

///Text of game rules.
///Multiline string literal just works.
///End of line in the code is simply and intuitively end of line in the string.
///The special character \ at the end of the line in code means that it is NOT the end of the line for the string.
///The escape sequence \n means end of line also. For doublequote simply \" .
const GAME_RULES:& str = "This game is for exactly 2 players.  
Both players must have the webpage simultaneously opened in the browser to allow communication.  
To start over just refresh the webpage.  
The first player clicks on 'Invite for play?' and broadcasts the message over WebSocket.  
He can choose different types of play: alphabet, animal,...  
Player2 then sees on the screen 'Click here to Accept play!', clicks it and sends the message back to Player1.  
The game starts with a grid of 8 randomly shuffled card pairs face down - 16 cards in all.  
On the screen under the grid are clear signals which player plays and which waits.  
Player1 flips over two cards with two clicks. The cards are accompanied by sounds and text on the screen.  
If the cards do not match, the other player clicks on 'Click here to Take your turn' and both cards are flipped back face down. Then it is his turn and he clicks to flip over his two cards.  
If the cards match, they are left face up permanently and the player receives a point. He continues to play, he opens the next two cards.  
The game is over when all the cards are permanently face up. It means that the sum of points is exactly 8.  
Click on \"Play again?\" to start the game over.  ";

///game description
const GAME_DESCRIPTION:& str = "Learning to use Rust Wasm/WebAssembly with Dodrio Virtual Dom and WebSockets communication - third iteration.";

///Render Component: The static parts can be cached easily.
pub struct RulesAndDescription {}

impl Render for RulesAndDescription {
    ///This rendering will be rendered and then cached . It will not be rerendered untill invalidation.
    ///In this case I don't need to invalidate because it is a static content.
    fn render<'a, 'bump>(&'a self, bump: &'bump Bump) -> Node<'bump>
    where
        'a: 'bump,
    {
        div(bump)
        .children([
            h4(bump)
            .children(text_with_br_newline(GAME_DESCRIPTION,bump))
            .finish(),
            h2(bump)
            .children([text(
                bumpalo::format!(in bump, "Memory game rules: {}", "").into_bump_str(),
            )])
            .finish(),
            h4(bump)
            .children(text_with_br_newline(GAME_RULES, bump))
            .finish(),
            h6(bump)
            .children([
                text(bumpalo::format!(in bump, "Learning Rust programming: {}", "").into_bump_str(),),
                a(bump)
                    .attr("href", "https://github.com/LucianoBestia/mem4_game")  
                    .attr("target","_blank")              
                    .children([text(bumpalo::format!(in bump, "https://github.com/LucianoBestia/mem4_game{}", "").into_bump_str(),)])
                    .finish(),
            ])
                .finish(),
        ])
        .finish()
    }
}
