use crate::abi::Message;

#[derive(Debug, Default)]
pub struct HandlerOutcome {
    pub messages: Vec<Message>,
}

