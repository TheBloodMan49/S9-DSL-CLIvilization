use openai::chat::{
    ChatCompletionDelta, ChatCompletionMessage, ChatCompletionMessageRole,
};
use openai::Credentials;
use std::default::Default;

pub struct AI {
    credentials: Credentials,
    model: &'static str,
    messages: Vec<ChatCompletionMessage>,
}

/// The AI needs the following tokens in env:
/// `OPENAI_KEY` &
/// `OPENAI_BASE_URL`
impl AI {
    pub fn new(model: &'static str) -> Self {
        AI {
            credentials: Credentials::from_env(),
            model,
            messages: Vec::new(),
        }
    }

    pub async fn send_message(&mut self, text: String) {
        let message = ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(text),
            ..Default::default()
        };

        self.messages.push(message);

        let chat_completion = ChatCompletionDelta::builder(self.model, self.messages.clone())
            .credentials(self.credentials.clone())
            .create()
            .await
            .unwrap();

        let returned_message = chat_completion.choices.first().unwrap().message.clone();

        self.messages.push(returned_message.clone());

        println!(
            "{:#?}: {}",
            returned_message.role,
            returned_message.content.unwrap().trim()
        );
    }
}