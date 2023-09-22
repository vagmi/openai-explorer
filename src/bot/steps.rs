use async_openai::{types::ChatCompletionRequestMessage, config::OpenAIConfig, Client};
use anyhow::{Result, anyhow};

mod simple_step;
mod function_step;

pub use simple_step::SimpleStepDefinition;
pub use function_step::{FunctionStepDefinition, Function, FunctionRegistry};


#[async_trait::async_trait]
pub trait ExecutableStep {
    async fn execute(&self, 
                     user_message: String, 
                     messages: Vec<ChatCompletionRequestMessage>, 
                     client: Client<OpenAIConfig>) -> Result<Vec<ChatCompletionRequestMessage>>;
}

pub enum StepDefinition<Fns: FunctionRegistry + Send + Sync> {
    Simple(SimpleStepDefinition),
    WithFunctions(FunctionStepDefinition<Fns>),
}

impl<Fns: FunctionRegistry + Send + Sync> StepDefinition<Fns> {
    pub fn user_message(&self) -> String {
        match self {
            Self::Simple(sd) => sd.user_message.clone(),
            _ => String::from("error")
        }
    }
}

#[async_trait::async_trait]
impl<Fns: FunctionRegistry + Send + Sync> ExecutableStep for StepDefinition<Fns> {
    async fn execute(&self, 
                     user_message: String, 
                     messages: Vec<ChatCompletionRequestMessage>, 
                     client: Client<OpenAIConfig>) -> Result<Vec<ChatCompletionRequestMessage>> {
        match self {
            Self::Simple(sd) => sd.execute(user_message, messages, client).await,
            Self::WithFunctions(fd) => fd.execute(user_message, messages, client).await,
        }
    }

}
