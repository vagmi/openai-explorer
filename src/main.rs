use async_openai::{Client, types::Role};
use async_trait::async_trait;
use bot::{Bot, SimpleStepDefinition, StepDefinition, FunctionStepDefinition, Function, FunctionRegistry};
use rustyline::DefaultEditor;
use serde_json::{json, Value};
use tracing_subscriber::EnvFilter;
use anyhow::Result;

mod bot;

#[derive(Clone)]
struct VenueBotFunctions {
}

#[async_trait]
impl FunctionRegistry for VenueBotFunctions {
    fn function_definitions(&self) -> Vec<Function>  {
        vec![ Function::new(String::from("get_locations"), 
                            String::from("Gets a list of cities and locations where Vivari operates."),
                            json!({"type": "object", "properties": {}}))]
    }

    async fn execute_function(&self, name:String, args:Value) -> Value {
        tracing::debug!("calling function named {:?} with args {:?}", name, args);
        if name == "get_locations" {
            json!([
                     {"id": 1, "city": "Fredericton", "name": "Planet Hatch Uptown"},
                     {"id": 2, "city": "Fredericton", "name": "Planet Hatch Downtown"},
                     {"id": 3, "city": "Moncton", "name": "Venn Innovations"},
            ])
        } else {
            json!({"error": "unknown function"})
        }
        
    }
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let mut rl = DefaultEditor::new().unwrap();
    let client = Client::new();


    // let bmc_prompt = include_str!("bmc_prompt.txt");
    // let value_prop_prompt = include_str!("value_proposition_prompt.txt");

    // let mut bot = Bot::new(String::from("consultant_gpt"), vec![
    //                        StepDefinition::Simple(SimpleStepDefinition::new(String::from(bmc_prompt), String::new())),
    //                        StepDefinition::Simple(
    //                            SimpleStepDefinition::new(String::from(value_prop_prompt), 
    //                                                      String::from("Can you please create a value proposition canvas for me?")))
    // ]);
    // let messages = bot.execute(String::from("Can you expand on the idea of creating a tool for prompt engineers to quickly expose their prompt as an API."), client).await.unwrap();


    let venue_bot_prompt = include_str!("venue_bot.txt");
    let venue_bot_functions = VenueBotFunctions{};
    let mut bot = Bot::new(String::from("venue_bot"), vec![
                           StepDefinition::WithFunctions(
                               FunctionStepDefinition::new(String::from(venue_bot_prompt), 
                                                           String::new(), 
                                                           venue_bot_functions
                                                           )),

    ]);

    loop {
        let input = rl.readline(">> ");
        match input {
            Ok(input) => {
                let messages = bot.execute(input, client.clone()).await.unwrap();
                //filter out the last assistant messages
                if let Some(msg) = messages.iter()
                    .filter(|m| {
                        m.role == Role::Assistant
                    }).last() {
                        println!("<< {}", msg.content.clone().map_or(String::from("No content"), |c| {c}));
                    }
            },
            Err(_) => {
                println!("Error");
                break;
            }
        }
    }

}
