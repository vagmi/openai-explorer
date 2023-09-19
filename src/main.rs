use async_openai::{Client, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessageArgs, Role}};
use bot::{Bot, StepDefinition};

mod bot;


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let client = Client::new();

    let bmc_prompt = include_str!("bmc_prompt.txt");
    let value_prop_prompt = include_str!("value_proposition_prompt.txt");
    
    let mut bot = Bot::new(String::from("consultant_gpt"), vec![
                           StepDefinition::new(String::from(bmc_prompt), String::new()) ,
                           StepDefinition::new(String::from(value_prop_prompt), 
                                               String::from("Can you please create a value proposition canvas for me?"))
    ]);
    let messages = bot.execute(String::from("Can you expand on the idea of creating a tool for prompt engineers to quickly expose their prompt as an API."), client).await.unwrap();
    

    // let bot_prompt = include_str!("sarcastic_bot_prompt.txt");
    // let mut bot = Bot::new(String::from("sarcastic"), vec![StepDefinition::new(String::from(bot_prompt), String::new())]);
    // let messages = bot.execute(String::from("Can you tell me the effect of atmospheric pressure on a liquid's boiling point?"), client).await.unwrap();
    for msg in messages {
        println!("{:?}", msg);
    }


}
