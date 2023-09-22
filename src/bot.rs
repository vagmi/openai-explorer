mod steps;

use anyhow::Result;
use async_openai::{types::ChatCompletionRequestMessage, Client, config::OpenAIConfig};

pub use self::steps::{StepDefinition, ExecutableStep, 
                      SimpleStepDefinition, 
                      FunctionStepDefinition, Function, FunctionRegistry};

pub struct Bot<Fns: FunctionRegistry + Send + Sync> {
    name: String,
    steps: Vec<StepDefinition<Fns>>,
    messages: Vec<ChatCompletionRequestMessage>,
}

impl<Fns: FunctionRegistry + Send + Sync> Bot<Fns> {
    pub fn new(name: String, steps: Vec<StepDefinition<Fns>>) -> Self {
        let messages = Vec::<ChatCompletionRequestMessage>::new();
        Self{name, steps, messages}
    }
    pub async fn execute(&mut self, user_message: String, client: Client<OpenAIConfig>) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut current_step = 0;

        for step in &self.steps {
            let msg = if current_step == 0 { user_message.clone() } else { step.user_message() };
            self.messages = step.execute(msg, self.messages.clone(), client.clone()).await?;

            current_step = current_step + 1
        }
        Ok(self.messages.clone())
    }
}




