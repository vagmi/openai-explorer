use async_openai::{types::{ChatCompletionRequestMessageArgs, Role, CreateChatCompletionRequestArgs, ChatCompletionRequestMessage, CreateChatCompletionRequest, ChatCompletionFunctionsArgs, ChatCompletionFunctions}, Client, config::OpenAIConfig};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::{Value, json};

use super::ExecutableStep;

pub struct FunctionStepDefinition<Fns>
where Fns: FunctionRegistry + Send + Sync {

    prompt: String,
    user_message: String,
    functions: Fns 
}

#[async_trait]
pub trait FunctionRegistry {
    fn function_definitions(&self) -> Vec<Function>;
    async fn execute_function(&self, name: String, args: Value) -> Value;
}

impl<Fns: FunctionRegistry + Send + Sync> FunctionStepDefinition<Fns> {
    pub fn new(prompt: String, user_message: String, functions: Fns) -> Self {
        Self{prompt, user_message, functions}
    }

    fn build_raw_request(&self, msgs: Vec<ChatCompletionRequestMessage>) -> Result<CreateChatCompletionRequest> {
        let functions = self.functions.function_definitions();
        let fns = functions.iter().map(|f| f.to_function_args()).collect::<Result<Vec<ChatCompletionFunctions>>>()?;
        tracing::debug!("functions {:?}", fns);
        tracing::debug!("messages {:?}", msgs);
        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(1024u16)
            .model("gpt-3.5-turbo-0613")
            .functions(fns)
            .messages(msgs)
            .function_call("auto")
            .build()?;
        Ok(request)
    }

    pub fn build_request(&self, role: Role, user_message: String, messages: Vec<ChatCompletionRequestMessage>) -> Result<CreateChatCompletionRequest> {
        let mut msgs = self.chat_messages(messages)?;
        msgs.push(ChatCompletionRequestMessageArgs::default()
                  .role(role)
                  .content(&user_message)
                  .build()?
                 );
        tracing::debug!("messages {:?}", msgs);
        self.build_raw_request(msgs)
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
impl<Fns: FunctionRegistry + Send + Sync> ExecutableStep  for FunctionStepDefinition<Fns> {
    async fn execute(&self, user_message: String, messages: Vec<ChatCompletionRequestMessage>, client: Client<OpenAIConfig>) -> Result<Vec<ChatCompletionRequestMessage>>  {
        let mut msgs = self.chat_messages(messages)?;
        let request = self.build_request(Role::User, user_message, msgs.clone())?;
        let mut resp = client.chat().create(request).await?;
        while let Some(choice) =  resp.clone().choices.first() {
            if let Some(finish_reason) = &choice.finish_reason {
                if finish_reason == "function_call" {
                    if let Some(fc) = &choice.message.function_call {
                        msgs.push(
                            ChatCompletionRequestMessageArgs::default()
                            .role(choice.message.role.clone())
                            .function_call(fc.clone())
                            .build()?);
                            let args = serde_json::from_str::<Value>(&fc.arguments)?;
                        let fc_content = self.functions.execute_function(fc.name.clone(), args.clone()).await;

                        msgs.push(ChatCompletionRequestMessageArgs::default()
                                  .role(Role::Function)
                                  .name(fc.name.clone())
                                  .content(fc_content.to_string())
                                  .build()?);
                        resp = client.chat().create(self.build_raw_request(msgs.clone())?).await?;
                    }
                } else {
                    if let Some(msg) = &choice.message.content {
                        msgs.push(
                            ChatCompletionRequestMessageArgs::default()
                            .role(choice.message.role.clone())
                            .content(String::from(msg))
                            .build()?);
                        break;
                    }
                }
            }
        }
        Ok(msgs)
    }
}



pub struct Function {
    name: String,
    description: String,
    args: Value
}

impl Function {
    pub fn new(name: String, description: String, args: Value) -> Self {
        Self{name, description, args}
    }
    pub fn to_function_args(&self) -> Result<ChatCompletionFunctions> {
        let args = ChatCompletionFunctionsArgs::default()
           .description(&self.description)
           .name(&self.name)
           .parameters(self.args.clone())
           .build()?;
        Ok(args)
    }

}
