#[derive(Copy, Clone, PartialEq, Eq, Debug, serde::Deserialize, serde::Serialize)]
pub enum State {
    OPEN,
    CLOSED,
}
