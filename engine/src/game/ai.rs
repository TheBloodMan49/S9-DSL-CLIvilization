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
            messages: vec![ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some("You are an AI playing a civilization like game in TUI\
                You will be given the rules.\
                You will be give a map of the world.\
                You will be given the output format.\
                For each turn you will be given the list of possible inputs.\
                You will have to select an input and output it as a json.\
                Rules:
                    - This is a human vs AI game.
                    - This is a turn based game.
                    - The human always plays before the AI (you).
                    - Each player has one city.
                    - To win you must destroy your opponent's city.
                    - Each turn you can select zero or multiple actions.
                    - To finish your turn you MUST say end.
                ".to_string()),
                ..Default::default()
            }],
        }
    }

    pub async fn send_message(&mut self, text: String) {
        let message = ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(text),
            ..Default::default()
        };

        self.messages.push(message);

        let chat_completion_res = ChatCompletionDelta::builder(self.model, self.messages.clone())
            .credentials(self.credentials.clone())
            .create()
            .await;

        let chat_completion = match chat_completion_res {
            Ok(c) => c,
            Err(e) => {
                log::error!("AI chat completion failed: {e}");
                return;
            }
        };

        let returned_message_opt = chat_completion.choices.first().map(|c| c.message.clone());
        let returned_message = if let Some(m) = returned_message_opt { m } else {
            log::warn!("AI chat completion returned no choices");
            return;
        };

        self.messages.push(returned_message.clone());

        log::debug!("AI response ({:?}): {}", returned_message.role, returned_message.content.as_deref().unwrap_or(""));
    }
}