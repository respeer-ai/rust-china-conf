pub mod errors;

#[derive(Debug, Default)]
pub struct HandlerOutcome {
    pub messages: Vec<Message>,
}

pub trait Handler {
    async fn handle(&mut self) -> Result<HandlerOutcome, HandlerError>,
}

pub struct HandlerFactory;

impl HandlerFactory {
    pub fn new(runtime: ContractRuntimeContext, state: mut impl StateInterface, op: &Operation) -> Result<Box<Handler>, HandlerError> {
        match op {

        }
    }
}
