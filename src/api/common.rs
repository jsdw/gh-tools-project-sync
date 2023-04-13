#[derive(Copy, Clone, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize)]
pub enum State {
    OPEN,
    CLOSED,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            State::OPEN => "State:OPEN",
            State::CLOSED => "State:CLOSED"
        };
        f.write_str(s)
    }
}