//! divfordebuggong.rs - information for debugging

//region: use, const
use crate::rootrenderingcomponent::RootRenderingComponent;

use dodrio::builder::text;
use dodrio::bumpalo::{self, Bump};
use dodrio::Node;
use typed_html::dodrio;
//endregion

///information for debugging
pub fn div_for_debugging<'a>(rrc: &'a RootRenderingComponent, bump: &'a Bump) -> Node<'a> {
    //for debugging only
    let text2 = bumpalo::format!(in bump, "debug: status: {}, ws_uid: {}",
    rrc.game_data.game_status,rrc.game_data.my_ws_uid)
    .into_bump_str();

    dodrio!(bump,
    <h4>
        {vec![text(text2)]}
    </h4>
    )
}
