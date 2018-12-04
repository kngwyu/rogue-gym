use input::System;

/// A representation of Ui transition
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum UiState {
    Dungeon,
    Mordal(MordalKind),
}

impl UiState {
    pub(crate) fn die(message: String) -> Self {
        UiState::Mordal(MordalKind::Grave(message.into_boxed_str()))
    }
}

/// mordals
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MordalKind {
    Grave(Box<str>),
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
            MordalKind::Grave(_) => match input {
                System::Cancel | System::Continue => MordalMsg::Quit,
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
