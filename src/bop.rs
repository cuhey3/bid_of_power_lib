use crate::bop::scenes::game_main::GameMainState;
use crate::bop::state::bind::get_binds;
use crate::bop::state::card_game_shared_state::{
    AttackTargetMessage, BidMessage, CardGamePlayer, Phase, PlayerState, UseCardMessage,
};
use crate::engine::application_types::StateType;
use crate::engine::state::{Primitives, References, State};
use crate::engine::Engine;
use crate::features::animation::Animation;
use crate::features::websocket::WebSocketWrapper;
use crate::svg::SharedElements;
use mechanism::card::Card;
use rand::Rng;
use scenes::title::TitleState;
use state::card_game_shared_state::CardGameSharedState;
use std::cell::RefCell;
use std::rc::Rc;

pub mod mechanism;
pub mod scenes;
pub mod state;

pub fn mount() -> Engine {
    let mut rng = rand::thread_rng();
    let random_number = rng.random::<u16>();
    let user_name = random_number.to_string();

    let rpg_shared_state = CardGameSharedState {
        players: vec![
            CardGamePlayer {
                player_name: "プレイヤー1".to_string(),
                game_start_is_approved: false,
                battle_is_viewed: false,
                own_card_list: vec![],
                player_state: PlayerState::init(),
            },
            CardGamePlayer {
                player_name: "プレイヤー2".to_string(),
                game_start_is_approved: false,
                battle_is_viewed: false,
                own_card_list: vec![],
                player_state: PlayerState::init(),
            },
        ],
        own_player_index: 0,
        cards_bid_on: vec![],
        bid_input: vec![BidMessage::init(), BidMessage::init(), BidMessage::init()],
        bid_scheduled_cards: Card::card_set_default(),
        temporary_bid_history: vec![],
        bid_history: vec![],
        use_card_input: UseCardMessage::empty(),
        use_card_history: vec![],
        attack_target_input: AttackTargetMessage::empty(),
        attack_target_history: vec![],
        initiatives_to_player_index: vec![0, 1],
        game_logs: vec![],
        turn: 0,
        phase_index: 0,
        phases: Phase::get_phases(),
        simple_binders: get_binds(),
        input_is_guard: false,
        consumed_seq_no: 0,
    };
    let mut shared_state = State {
        user_name: user_name.to_owned(),
        to_send_channel_messages: vec![],
        elements: SharedElements::new(),
        interrupt_animations: vec![vec![Animation::always_blink()]],
        state_type: StateType::BoPShared(rpg_shared_state),
        primitives: Primitives {
            scene_index: 0,
            requested_scene_index: 0,
            map_index: 0,
            requested_map_index: 0,
        },
        references: Rc::new(RefCell::new(References {
            has_block_message: false,
            has_continuous_message: false,
        })),
        is_request_matching: false,
        is_matched: false,
        keep_connection_request: false,
    };

    let mut scenes = vec![
        TitleState::create_title_scene(&mut shared_state),
        GameMainState::create_game_main_scene(&mut shared_state),
    ];
    let init_func = scenes[0].init_func;
    init_func(&mut scenes[0], &mut shared_state);
    let web_socket_wrapper =
        WebSocketWrapper::new(shared_state.user_name.to_owned(), "bop".to_string());
    Engine::new(shared_state, scenes, web_socket_wrapper)
}
