#[derive(Clone, Debug)]
pub struct PlayerState {
    pub max_hp: u32,
    pub current_hp: u32,
    pub attack_point: u32,
    pub defence_point: u32,
    pub current_money_amount: u32,
    pub estimated_money_amount: u32,
}

impl PlayerState {
    pub fn init() -> PlayerState {
        PlayerState {
            max_hp: 50,
            current_hp: 50,
            attack_point: 10,
            defence_point: 5,
            current_money_amount: 5,
            estimated_money_amount: 3,
        }
    }
}
