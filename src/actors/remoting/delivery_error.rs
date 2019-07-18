#[derive(Debug)]
pub struct DeliveryError {
    pub reason: DeliveryErrorReason
}

#[derive(Debug, Eq, PartialEq)]
pub enum DeliveryErrorReason {
    SerializationError,
    ConnectionLost,
    NetworkError
}
