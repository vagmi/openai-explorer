use anyhow::{Result, anyhow};
use std::iter::Extend;
use async_openai::{types::{
    CreateChatCompletionRequest, ChatCompletionRequestMessageArgs,
    ChatCompletionRequestMessage,
    CreateChatCompletionRequestArgs, Role}, Client, config::OpenAIConfig};

/*
 * {"consultant_bot": {
 *   "steps": [
     *   {
 *         "prompt": ""
     *   }, {
     *     "prompt": ""
     *     "user_message: "build me a value proposition canvas"
     *  
     *   }
 *   ],
    }
    }
 */

pub struct Bot {
    name: String,
    steps: Vec<StepDefinition>,
    current_step: u8,
}

impl Bot {
    pub fn new(name: String, steps: Vec<StepDefinition>) -> Self {
        Self{name, steps, current_step: 0}
    }
    pub async fn execute(&mut self, user_message: String, client: Client<OpenAIConfig>) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut messages = Vec::<ChatCompletionRequestMessage>::new();

        for step in &self.steps {
            let msg = if self.current_step == 0 { user_message.clone() } else { step.user_message.clone() };
            let mut current_step = Step {
                step_definition: step.clone(),
                messages: messages.clone()
            };
            messages = current_step.execute(msg, messages, client.clone()).await?;

            self.current_step = self.current_step + 1
        }
        Ok(messages)
    }
}

#[derive(Debug, Clone)]
pub struct StepDefinition {
    prompt: String,
    user_message: String,
}

impl StepDefinition {
    pub fn new(prompt: String, user_message: String) -> Self {
        Self { prompt, user_message }
    }
}

struct Step {
    step_definition: StepDefinition,
    messages: Vec<ChatCompletionRequestMessage>
}


impl Step {
    fn build_request(&self, user_message: String, messages: Vec<ChatCompletionRequestMessage>) -> Result<CreateChatCompletionRequest> {
        let mut msgs = Vec::<ChatCompletionRequestMessage>::new();

        msgs.push(ChatCompletionRequestMessageArgs::default()
                  .role(Role::System)
                  .content(&self.step_definition.prompt)
                  .build()?);
        msgs.extend(messages);
        msgs.push(ChatCompletionRequestMessageArgs::default()
                  .role(Role::User)
                  .content(&user_message)
                  .build()?
                 );
        tracing::debug!("messages {:?}", msgs);
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model("gpt-3.5-turbo")
            .messages(msgs).build()?;
        Ok(request)
    }

    async fn execute(&mut self, user_message: String, messages: Vec<ChatCompletionRequestMessage>, client: Client<OpenAIConfig>) -> Result<Vec<ChatCompletionRequestMessage>>  {
        let request = self.build_request(user_message, messages)?;
        let resp = client.chat().create(request).await?;
        match resp.choices.first() {
            Some(choice) => { 
                match &choice.message.content {
                    Some(msg) => {
                        self.messages.push(ChatCompletionRequestMessageArgs::default().role(choice.message.role.clone()).content(String::from(msg)).build()?);
                        Ok(())
                    },
                    None => {Ok(())}
                }

            },
            None => {Err(anyhow!("unable to execute step"))}
        }?;
        Ok(self.messages.clone())
    }
}

