use crate::bop::scenes::battle::BattleState;
use crate::bop::scenes::event::EventState;
use crate::bop::scenes::field::FieldState;
use crate::bop::scenes::game_main::GameMainState;
use crate::bop::scenes::menu::MenuState;
use crate::bop::scenes::title::TitleState;
use crate::bop::state::card_game_shared_state::CardGameSharedState;

pub enum StateType {
    BoPShared(CardGameSharedState),
    TBDStateType,
}

pub enum SceneType {
    BoPTitle(TitleState),
    BoPGameMain(GameMainState),
    RPGEvent(EventState),
    RPGField(FieldState),
    RPGBattle(BattleState),
    RPGMenu(MenuState),
}
