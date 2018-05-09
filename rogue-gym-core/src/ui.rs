use input;
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum UiState {
    Dungeon,
    Mordal(MordalKind),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum MordalKind {
    Quit,
}

impl MordalKind {
    pub(crate) fn process(&mut self, input: input::System) -> MordalMsg {
        // STUB!
        MordalMsg::A
    }
}

pub(crate) enum MordalMsg {
    A,
}
