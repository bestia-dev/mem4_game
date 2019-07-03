//! statusplaybefore1card.rs - code flow from this status

//region: use
use crate::rootrenderingcomponent::RootRenderingComponent;
use mem4_common::GameStatus;
//endregion

//div_grid_container is in divgridcontainer

//div_grid_item_on_click is in divgridcontainer

///msg player click
pub fn on_msg_player_click_1_card(
    rrc: &mut RootRenderingComponent,
    game_status: GameStatus,
    card_index: usize,
) {
    rrc.game_data.game_status = game_status;
    if rrc.game_data.game_status.as_ref() == GameStatus::PlayBefore1Card.as_ref() {
        rrc.game_data.card_index_of_first_click = card_index;
        rrc.card_on_click_1_card();
    } else {
        panic!("this else should never be reached");
    }
}