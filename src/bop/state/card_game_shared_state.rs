use crate::bop::mechanism::card::Card;
use crate::bop::mechanism::card::CardKind::*;
use crate::bop::mechanism::player_state::PlayerState;
use crate::bop::state::phase::PhaseType::*;
#[derive(Clone)]
pub struct CardGamePlayer {
    pub player_name: String,
    pub game_start_is_approved: bool,
    pub battle_is_viewed: bool,
    pub own_card_list: Vec<Card>,
    pub player_state: PlayerState,
}

#[derive(Clone)]
pub struct GameLog {
    pub turn: usize,
    pub log_type: LogType,
}

#[derive(Clone)]
pub enum LogType {
    Joined(usize),
    BidSuccessful(usize),
    InitiativeChanged(usize),
    UseCard(usize),
    AttackTarget(usize, u32),
    GameEnd(usize),
}

