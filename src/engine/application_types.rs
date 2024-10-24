use crate::bop::scenes::game_main::GameMainState;
use crate::bop::scenes::title::TitleState;
use crate::bop::state::message::CardGameSharedState;

pub enum StateType {
    BoPShared(CardGameSharedState),
    TBDStateType,
}

pub enum SceneType {
    BoPTitle(TitleState),
    BoPGameMain(GameMainState),
}
