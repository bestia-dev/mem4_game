//! statusplaybefore1stcard.rs - code flow from this status

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
pub fn div_click_1st_card<'a, 'bump>(
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

/// on click
pub fn on_click_1st_card(rrc: &mut RootRenderingComponent, this_click_card_index: usize) {
    rrc.game_data.card_index_of_first_click = this_click_card_index;
    //region: send WsMessage over WebSocket
    websocketcommunication::ws_send_msg(
        &rrc.game_data.ws,
        &WsMessage::PlayerClick1stCard {
            my_ws_uid: rrc.game_data.my_ws_uid,
            players: unwrap!(
                serde_json::to_string(&rrc.game_data.players),
                "serde_json::to_string(&game_data.players)",
            ),
            card_index: rrc.game_data.card_index_of_first_click,
            game_status: rrc.game_data.game_status.clone(),
        },
    );
    //endregion
    card_click_1st_card(rrc);
}

///on click
pub fn card_click_1st_card(rrc: &mut RootRenderingComponent) {
    //flip the card up
    unwrap!(
        rrc.game_data
            .vec_cards
            .get_mut(rrc.game_data.card_index_of_first_click),
        "error this_click_card_index"
    )
    .status = CardStatusCardFace::UpTemporary;
    rrc.check_invalidate_for_all_components();
}

///msg player click
pub fn on_msg_player_click_1st_card(
    rrc: &mut RootRenderingComponent,
    game_status: GameStatus,
    card_index: usize,
) {
    rrc.game_data.game_status = game_status;
    if rrc.game_data.game_status.as_ref() == GameStatus::PlayBefore1stCard.as_ref() {
        rrc.game_data.card_index_of_first_click = card_index;
        card_click_1st_card(rrc);
    } else {
        panic!("this else should never be reached");
    }
}