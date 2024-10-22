use crate::bop::scenes::game_main::GameMainState;
use crate::bop::scenes::title::TitleState;
use crate::bop::state::card_game_shared_state::CardGameSharedState;

pub enum StateType {
    BoPShared(CardGameSharedState),
    TBDStateType,
}

pub enum SceneType {
    BoPTitle(TitleState),
    BoPGameMain(GameMainState),
}
