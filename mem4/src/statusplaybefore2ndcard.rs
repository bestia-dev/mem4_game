//! statusplaybefore2ndcard.rs - code flow from this status

//region: use
use crate::gamedata::CardStatusCardFace;
use crate::rootrenderingcomponent::RootRenderingComponent;
use crate::websocketcommunication;
use mem4_common::{GameStatus, WsMessage};

use dodrio::builder::text;
use dodrio::bumpalo::{self, Bump};
use dodrio::Node;
use typed_html::dodrio;
//endregion

///render Play or Wait
pub fn div_click_2nd_card<'a, 'bump>(
    root_rendering_component: &'a RootRenderingComponent,
    bump: &'bump Bump,
) -> Node<'bump>
where
    'a: 'bump,
{
    if root_rendering_component.game_data.my_player_number
        == root_rendering_component.game_data.player_turn
    {
        dodrio!(bump,
        <div class="div_clickable">
            <h2 id= "ws_elem" style= "color:orange;">
                {vec![text(bumpalo::format!(in bump, "Play player{} !", root_rendering_component.game_data.player_turn).into_bump_str())]}
            </h2>
        </div>
        )
    } else {
        //return wait for the other player
        dodrio!(bump,
        <h2 id="ws_elem" style= "color:red;">
            {vec![text(bumpalo::format!(in bump, "Wait for player{} !", root_rendering_component.game_data.player_turn).into_bump_str())]}
        </h2>
        )
    }
}

//div_grid_container() is in divgridcontainer.rs

///on click
pub fn on_click_2nd_card(rrc: &mut RootRenderingComponent, this_click_card_index: usize) {
    rrc.game_data.card_index_of_second_click = this_click_card_index;
    card_click_2nd_card(rrc, "on_click");
}

///on second click
///The on click event passed by JavaScript executes all the logic
///and changes only the fields of the Card Grid struct.
///That struct is the only permanent data storage for later render the virtual dom.
pub fn card_click_2nd_card(rrc: &mut RootRenderingComponent, caller: &str) {
    //region: send WsMessage over WebSocket
    //don't send if the caller is on_msg
    if caller == "on_click" {
        websocketcommunication::ws_send_msg(
            &rrc.game_data.ws,
            &WsMessage::PlayerClick2ndCard {
                my_ws_uid: rrc.game_data.my_ws_uid,
                players: unwrap!(
                    serde_json::to_string(&rrc.game_data.players),
                    "serde_json::to_string(&game_data.players)",
                ),
                card_index: rrc.game_data.card_index_of_second_click,
                game_status: rrc.game_data.game_status.clone(),
            },
        );
    }
    //endregion
    //flip the card up
    unwrap!(
        rrc.game_data
            .card_grid_data
            .get_mut(rrc.game_data.card_index_of_second_click),
        "error this_click_card_index"
    )
    .status = CardStatusCardFace::UpTemporary;

    //if the cards match, player get one point and continues another turn
    if unwrap!(
        rrc.game_data
            .card_grid_data
            .get(rrc.game_data.card_index_of_first_click),
        "error game_data.card_index_of_first_click"
    )
    .card_number_and_img_src
        == unwrap!(
            rrc.game_data
                .card_grid_data
                .get(rrc.game_data.card_index_of_second_click),
            "error game_data.card_index_of_second_click"
        )
        .card_number_and_img_src
    {
        //give points
        unwrap!(
            rrc.game_data
                .players
                .get_mut(unwrap!(rrc.game_data.player_turn.checked_sub(1))),
            "rrc.game_data.players.get_mu(rrc.game_data.player_turn - 1)"
        )
        .points += 1;

        // the two cards matches. make them permanent FaceUp
        let x1 = rrc.game_data.card_index_of_first_click;
        let x2 = rrc.game_data.card_index_of_second_click;
        unwrap!(
            rrc.game_data.card_grid_data.get_mut(x1),
            "error game_data.card_index_of_first_click"
        )
        .status = CardStatusCardFace::UpPermanently;
        unwrap!(
            rrc.game_data.card_grid_data.get_mut(x2),
            "error game_data.card_index_of_second_click"
        )
        .status = CardStatusCardFace::UpPermanently;
        //if the sum of points is number of card/2, the game is over
        let mut point_sum = 0;
        for x in &rrc.game_data.players {
            point_sum += x.points;
        }
        if unwrap!(rrc.game_data.card_grid_data.len().checked_div(2)) == point_sum {
            //The game is over and the question Play again?
            rrc.game_data.game_status = GameStatus::PlayAgain;
            //send message
            unwrap!(
                rrc.game_data.ws.send_with_str(
                    &serde_json::to_string(&WsMessage::PlayAgain {
                        my_ws_uid: rrc.game_data.my_ws_uid,
                        players: unwrap!(
                            serde_json::to_string(&rrc.game_data.players),
                            "serde_json::to_string(&rrc.game_data.players)"
                        ),
                    })
                    .expect("error sending PlayAgain"),
                ),
                "Failed to send PlayAgain"
            );
        } else {
            //the same payer continue to play
            rrc.game_data.game_status = GameStatus::PlayBefore1stCard;
        }
    } else {
        //if cards don't match
        rrc.game_data.game_status = GameStatus::TakeTurnBegin;
        //region: send WsMessage over WebSocket
        //don't send if it is called from on_msg
        if caller == "on_click" {
            websocketcommunication::ws_send_msg(
                &rrc.game_data.ws,
                &WsMessage::TakeTurnBegin {
                    my_ws_uid: rrc.game_data.my_ws_uid,
                    players: unwrap!(
                        serde_json::to_string(&rrc.game_data.players),
                        "serde_json::to_string(&game_data.players)",
                    ),
                    card_index: rrc.game_data.card_index_of_first_click,
                    game_status: rrc.game_data.game_status.clone(),
                },
            );
        }
        //endregion
        //now all the players are calculating the status of the game.
        //This is not ok. Only the active player should calculate and send a message to all others.
    }
    rrc.check_invalidate_for_all_components();
}
///msg player click
pub fn on_msg_player_click_2nd_card(
    rrc: &mut RootRenderingComponent,
    game_status: GameStatus,
    card_index: usize,
) {
    rrc.game_data.game_status = game_status;
    if rrc.game_data.game_status.as_ref() == GameStatus::PlayBefore2ndCard.as_ref() {
        rrc.game_data.card_index_of_second_click = card_index;
        card_click_2nd_card(rrc, "on_msg");
    } else {
        panic!("this else must never be reached!");
    }
}

///msg player click
pub fn on_msg_play_again(rrc: &mut RootRenderingComponent) {
    rrc.game_data.game_status = GameStatus::PlayAgain;
}
