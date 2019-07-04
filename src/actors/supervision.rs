//! Structs used for supervision

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum SupervisionStrategy {
    Resume,
    Stop,
    Restart,
    Escalate
}