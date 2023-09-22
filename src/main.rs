use std::{fmt::format, collections::HashMap};

use async_openai::{Client, types::Role};
use async_trait::async_trait;
use bot::{Bot, SimpleStepDefinition, StepDefinition, FunctionStepDefinition, Function, FunctionRegistry};
use reqwest::header::AUTHORIZATION;
use rustyline::DefaultEditor;
use serde_json::{json, Value};
use strfmt::strfmt;
use tracing_subscriber::EnvFilter;
use anyhow::Result;

mod bot;

#[derive(Clone)]
struct VenueBotFunctions {
    base_url: String,
}

impl VenueBotFunctions {
    pub async fn get_locations(&self) -> Result<Value> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/locations", self.base_url))
            .send()
            .await?;
        let body: Value = res.json().await?;
        Ok(body)
    }

    pub async fn get_facility_in_location(&self, location_id: u64) -> Result<Value> {
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/locations/{}/facilities", self.base_url, location_id))
            .send()
            .await?;
        let body: Value = res.json().await?;
        Ok(body)
    }

    pub async fn get_bookings(&self, token: String) -> Result<Value> {
        tracing::debug!("The auth token received is {:?}", token);
        let client = reqwest::Client::new();
        let res = client.get(format!("{}/bookings", self.base_url))
            .bearer_auth(token)
            .send()
            .await?;
        let body: Value = res.json().await?;
        Ok(body)
    }
}

#[async_trait]
impl FunctionRegistry for VenueBotFunctions {
    fn function_definitions(&self) -> Vec<Function>  {
        vec![ 
            Function::new(String::from("get_locations"), 
                            String::from("Gets a list of cities and locations where Vivari operates."),
                            json!({"type": "object", "properties": {}})),
            Function::new(String::from("get_facility_in_location"), 
                            String::from("Gets a list of Facilities in a specific location. Each facility a seating capacity and a list of equipments in it. For eg., Project, TV, Whiteboard etc"),
                            json!({"type": "object", "properties": {"location_id": {"type": "number"}}})),
            Function::new(String::from("get_bookings"), 
                            String::from("Get a list of bookings for the user. Each booking has a start time, end time, location and a facility"),
                            json!({"type": "object", "properties": {"token": {"type": "string"}}})),
             
        ]
    }

    async fn execute_function(&self, name:String, args:Value) -> Value {
        tracing::debug!("calling function named {:?} with args {:?}", name, args);
        if name == "get_locations" {
            match self.get_locations().await {
                Ok(locations) => return locations,
                Err(e) => {
                    tracing::error!("error calling get_locations: {:?}", e);
                    return json!({"error": "error calling get_locations"});
                }
            }
        } else if name == "get_facility_in_location" {
            let location_id = args["location_id"].as_u64().unwrap_or(0);
            match self.get_facility_in_location(location_id).await {
                Ok(facilities) => return facilities,
                Err(e) => {
                    tracing::error!("error calling get_facitilty for location: {:?}", e);
                    return json!({"error": "error calling get_facility for location"});
                }
            }
        } else if name == "get_bookings" {
            let token = args["token"].as_str().unwrap_or("").to_string();
            match self.get_bookings(token).await {
                Ok(bookings) => return bookings,
                Err(e) => {
                    tracing::error!("error calling get_bookings: {:?}", e);
                    return json!({"error": "error calling get_bookings"});
                }
            }
            
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

    let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOjEsIm5hbWUiOiJKb2huIERvZSIsImlhdCI6MTUxNjIzOTAyMn0.pOkegDyjuCYBoGyMO3q91JzPwLYQIrVVPgk_w3ffEhs";
    let mut vars = HashMap::new();
    vars.insert(String::from("token"), token.to_string());

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
    let prompt = strfmt(venue_bot_prompt, &vars).unwrap();
    let venue_bot_functions = VenueBotFunctions{base_url: String::from("http://localhost:3000/api")};
    let mut bot = Bot::new(String::from("venue_bot"), vec![
                           StepDefinition::WithFunctions(
                               FunctionStepDefinition::new(String::from(prompt), 
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
