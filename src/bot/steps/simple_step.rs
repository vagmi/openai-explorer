use async_openai::{Client, config::OpenAIConfig};

use async_openai::types::{
    CreateChatCompletionRequestArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestMessageArgs, 
    Role, CreateChatCompletionRequest};
use anyhow::{Result, anyhow};

use super::ExecutableStep;

#[derive(Debug, Clone)]
pub struct SimpleStepDefinition {
    pub prompt: String,
    pub user_message: String,
}

impl SimpleStepDefinition {

    pub fn new(prompt: String, user_message: String) -> Self {
        Self { prompt, user_message }
    }

    fn build_request(&self, user_message: String, messages: Vec<ChatCompletionRequestMessage>) -> Result<CreateChatCompletionRequest> {
        let mut msgs = self.chat_messages(messages)?;
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

    pub fn chat_messages(&self, messages: Vec<ChatCompletionRequestMessage>) -> Result<Vec<ChatCompletionRequestMessage>> {
        let mut msgs = Vec::<ChatCompletionRequestMessage>::new();

        msgs.push(ChatCompletionRequestMessageArgs::default()
                  .role(Role::System)
                  .content(&self.prompt)
                  .build()?);
        msgs.extend(messages);
        Ok(msgs)
    }
}



#[async_trait::async_trait]
impl ExecutableStep  for SimpleStepDefinition {
    async fn execute(&self, user_message: String, messages: Vec<ChatCompletionRequestMessage>, client: Client<OpenAIConfig>) -> Result<Vec<ChatCompletionRequestMessage>>  {
        let mut msgs = self.chat_messages(messages)?;
        let request = self.build_request(user_message, msgs.clone())?;
        let resp = client.chat().create(request).await?;
        match resp.choices.first() {
            Some(choice) => { 
                match &choice.message.content {
                    Some(msg) => {
                        msgs.push(ChatCompletionRequestMessageArgs::default().role(choice.message.role.clone()).content(String::from(msg)).build()?);
                        Ok(())
                    },
                    None => {Ok(())}
                }

            },
            None => {Err(anyhow!("unable to execute step"))}
        }?;
        Ok(msgs)
    }
}

