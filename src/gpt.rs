use minreq;
use serde::{Deserialize, Serialize};
use std::error::Error;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_MODEL: &str = "gpt-3.5-turbo";
const MAX_TOKENS: usize = 4096;
const TEMPERATURE: f32 = 0.2;

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    choices: Vec<Choice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

impl Message {
    fn new(role: &str, prompt: &str) -> Self {
        Self {
            role: role.to_string(),
            content: prompt.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Prompt<'a> {
    model: &'a str,
    messages: [Message; 2],
    temperature: f32,
}

impl<'a> Prompt<'a> {
    fn new(prompt: &'a str) -> Self {
        Self { model: OPENAI_MODEL,
            messages: [
            Message::new("system", "I want you to act as a coding assistant. I will provide prompts specifying a program or data structure definition. Please only reply with the code output. Do not include markdown symbols around the code block. Please make the code ready to paste into an editor. Do not write explanations."),
            Message::new("user", prompt)
            ],
            temperature: TEMPERATURE
        }
    }
}

pub struct GPTClient<'a> {
    api_key: &'a str,
    url: &'a str,
}

impl<'a> GPTClient<'a> {
    pub fn new(api_key: &'a str) -> Self {
        GPTClient {
            api_key,
            url: OPENAI_API_URL,
        }
    }

    pub fn prompt(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        if prompt.len() >= MAX_TOKENS {
            return Err(format!("Prompt cannot exceed length of {MAX_TOKENS} tokens").into());
        }

        let p = Prompt::new(prompt);

        let response: Response = minreq::post(self.url)
            .with_timeout(120)
            .with_header("Authorization", format!("Bearer {}", self.api_key))
            .with_json(&p)?
            .send()?
            .json::<_>()?;

        Ok(response.choices[0].message.content.to_string())
    }
}
