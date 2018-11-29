use input::System;

/// A representation of Ui transition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum UiState {
    Dungeon,
    Mordal(MordalKind),
}

/// mordals
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MordalKind {
    Inventory,
    Quit,
}

impl MordalKind {
    pub fn process(&mut self, input: System) -> MordalMsg {
        match self {
            MordalKind::Quit => match input {
                System::Cancel | System::No => MordalMsg::Cancel,
                System::Yes => MordalMsg::Quit,
                _ => MordalMsg::None,
            },
            MordalKind::Inventory => match input {
                System::Cancel | System::Continue => MordalMsg::Cancel,
                _ => MordalMsg::None,
            },
        }
    }
}

pub enum MordalMsg {
    Quit,
    Save,
    Cancel,
    None,
}
