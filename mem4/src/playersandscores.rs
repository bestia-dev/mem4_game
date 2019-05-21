//! file and module for playersandscores
use crate::gamedata::GameData;

use dodrio::builder::*;
use dodrio::bumpalo::{self, Bump};
use dodrio::{Node, Render};

///Render Component: player score
///Its private fields are a cache copy from `game_data` fields.
///They are used for rendering
///and for checking if the data has changed to invalidate the render cache.
pub struct PlayersAndScores {
    ///whose turn is now:  player 1 or 2
    player_turn: usize,
    ///player1 points
    player1_points: usize,
    ///player2 points
    player2_points: usize,
    ///What player am I
    this_machine_player_number: usize,
}

impl PlayersAndScores {
    ///constructor
    pub const fn new() -> Self {
        PlayersAndScores {
            player1_points: 0,
            player2_points: 0,
            this_machine_player_number: 0, //unknown until WantToPlay+Accept
            player_turn: 0,
        }
    }
    ///copies the data from game data to internal cache
    /// internal fiels are used to render component
    pub fn update_intern_cache(&mut self, game_data: &GameData) -> bool {
        let mut is_invalidated;
        is_invalidated = false;
        if self.player1_points != game_data.player1_points {
            self.player1_points = game_data.player1_points;
            is_invalidated = true;
        }
        if self.player2_points != game_data.player2_points {
            self.player2_points = game_data.player2_points;
            is_invalidated = true;
        }

        if self.this_machine_player_number != game_data.this_machine_player_number {
            self.this_machine_player_number = game_data.this_machine_player_number;
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
    ///It is ivalidate, when the points change.
    ///html element to with scores for 2 players
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
            .children([
                div(bump)
                    .attr("class", "grid_item")
                    .attr(
                        "style",
                        bumpalo::format!(in bump,"text-align: left;color:{};text-decoration:{}",
                            if self.player_turn==1 {"green"} else {"red"},
                            if self.this_machine_player_number==1 {"underline"} else {"none"}
                        )
                        .into_bump_str(),
                    )
                    .children([text(
                        bumpalo::format!(in bump, "player1: {}",self.player1_points)
                            .into_bump_str(),
                    )])
                    .finish(),
                div(bump)
                    .attr("class", "grid_item")
                    .attr("style", "text-align: center;")
                    .children([text("")])
                    .finish(),
                div(bump)
                    .attr("class", "grid_item")
                    .attr(
                        "style",
                        bumpalo::format!(in bump,"text-align: right;color:{};text-decoration:{}",
                            if self.player_turn==2 {"green"} else {"red"},
                            if self.this_machine_player_number==2 {"underline"} else {"none"}
                        )
                        .into_bump_str(),
                    )
                    .children([text(
                        bumpalo::format!(in bump, "player2: {}",self.player2_points)
                            .into_bump_str(),
                    )])
                    .finish(),
            ])
            .finish()
    }
}
