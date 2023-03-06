use super::util;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, HeaderValue},
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{error::Error, io::Read};

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const OPENAI_MODEL: &str = "gpt-3.5-turbo";
const MAX_TOKENS: usize = 4096;
const TEMPERATURE: f32 = 0.2;

type BoxResult<T> = Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

impl<'a> Message<'a> {
    fn new(role: &'a str, prompt: &'a str) -> Self {
        Self {
            role,
            content: prompt,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Prompt<'a> {
    model: &'a str,
    messages: [Message<'a>; 2],
    temperature: f32,
}

impl<'a> Prompt<'a> {
    fn new(prompt: &'a str) -> Self {
        Self { model: OPENAI_MODEL,
            messages: [
            Message::new("system", "I want you to act as a coding assistant. I will provide prompts specifying a program or data structure definition. Please only reply with the raw code output and nothing else. Do not include any markdown symbols around the code block. Please make the code ready to paste into an editor. Do not write explanations."),
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

    pub fn prompt(&self, prompt: &str) -> BoxResult<String> {
        if prompt.len() >= MAX_TOKENS {
            return Err(format!(
                "Prompt cannot exceed length of {} characters",
                MAX_TOKENS - 1
            )
            .into());
        }

        let client = Client::new();

        let p = Prompt::new(prompt);
        let body = serde_json::to_string(&p)?;

        let mut auth = String::from("Bearer ");
        auth.push_str(self.api_key);

        let mut headers = HeaderMap::new();
        headers.insert("Authorization", HeaderValue::from_str(auth.as_str())?);
        headers.insert("Content-Type", HeaderValue::from_str("application/json")?);

        let mut res = client.post(self.url).body(body).headers(headers).send()?;

        let mut response_body = String::new();
        res.read_to_string(&mut response_body)?;

        let json_object: Value = serde_json::from_str(&response_body)?;
        let answer = json_object["choices"][0]["message"]["content"].as_str();

        match answer {
            Some(a) => Ok(String::from(a)),
            None => {
                util::pretty_print(&response_body, "json");
                Err(format!("JSON parse error").into())
            }
        }
    }
}
