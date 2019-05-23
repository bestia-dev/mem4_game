//! file and module for playersandscores
use crate::gamedata::GameData;

use dodrio::builder::*;
use dodrio::bumpalo::{self, Bump};
use dodrio::{Node, Render};
use wasm_bindgen::prelude::*;
use web_sys::console;

///Render Component: player score
///Its private fields are a cache copy from `game_data` fields.
///They are used for rendering
///and for checking if the data has changed to invalidate the render cache.
pub struct PlayersAndScores {
    ///whose turn is now:  player 1 or 2
    player_turn: usize,
    ///my points
    my_points: usize,
    ///What player am I
    my_player_number: usize,
}

impl PlayersAndScores {
    ///constructor
    pub const fn new() -> Self {
        PlayersAndScores {
            my_points: 0,
            my_player_number: 1,
            player_turn: 0,
        }
    }
    ///copies the data from game data to internal cache
    /// internal fiels are used to render component
    pub fn update_intern_cache(&mut self, game_data: &GameData) -> bool {
        console::log_1(&JsValue::from_str(&format!(
            "update_intern_cache  my_player_number {}",
            &game_data.my_player_number
        )));

        let mut is_invalidated;
        is_invalidated = false;
        if game_data.my_player_number > 0
            && !game_data.players.is_empty()
            && game_data.players.len() >= game_data.my_player_number - 1
            && self.my_points != game_data.players[game_data.my_player_number - 1].points
        {
            self.my_points = game_data.players[game_data.my_player_number - 1].points;
            is_invalidated = true;
        }
        if self.my_player_number != game_data.my_player_number {
            self.my_player_number = game_data.my_player_number;
            is_invalidated = true;
        }
        if self.player_turn != game_data.player_turn {
            self.player_turn = game_data.player_turn;
            is_invalidated = true;
        }
        is_invalidated
    }
}

impl Render for PlayersAndScores {
    ///This rendering will be rendered and then cached . It will not be rerendered untill invalidation.
    ///It is invalidate, when the points change.
    ///html element with 1 score for this players
    fn render<'a, 'bump>(&'a self, bump: &'bump Bump) -> Node<'bump>
    where
        'a: 'bump,
    {
        //return
        div(bump)
            .attr("class", "grid_container_players")
            .attr(
                "style",
                bumpalo::format!(in bump, "grid-template-columns: auto auto auto;{}","")
                    .into_bump_str(),
            )
            .children([div(bump)
                .attr("class", "grid_item")
                .attr(
                    "style",
                    bumpalo::format!(in bump,"text-align: center;color:{};",
                        if self.player_turn==self.my_player_number {"green"} else {"red"}
                    )
                    .into_bump_str(),
                )
                .children([text(
                    bumpalo::format!(in bump, "player{}: {}",self.my_player_number, self.my_points)
                        .into_bump_str(),
                )])
                .finish()])
            .finish()
    }
}
