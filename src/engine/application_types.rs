use crate::bop::scenes::game_main::GameMainState;
use crate::bop::scenes::title::TitleState;
use crate::bop::state::bop_shared_state::BoPSharedState;

pub enum StateType {
    BoPShared(BoPSharedState),
    TBDStateType,
}

pub enum SceneType {
    BoPTitle(TitleState),
    BoPGameMain(GameMainState),
}
