use openai::chat::{
    ChatCompletionDelta, ChatCompletionMessage, ChatCompletionMessageRole,
};
use openai::Credentials;
use std::default::Default;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use crate::game::AiView;
use crate::game::state::Popup;

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

    /// Send a message to the LLM and return the content (if any)
    pub async fn send_message(&mut self, text: String) -> Option<String> {
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
                log::error!("AI chat completion failed: {}", e);
                return None;
            }
        };

        let returned_message_opt = chat_completion.choices.first().map(|c| c.message.clone());
        let returned_message = match returned_message_opt {
            Some(m) => m,
            None => {
                log::warn!("AI chat completion returned no choices");
                return None;
            }
        };

        self.messages.push(returned_message.clone());

        let content = returned_message.content.as_deref().map(|s| s.trim().to_string());
        log::debug!("AI response ({:?}): {}", returned_message.role, content.as_deref().unwrap_or(""));
        content
    }
}

// ===== LLM-backed Ai adapter =====

enum LlmRequest {
    SelectAction(AiView, usize, Sender<Option<String>>),
    SelectPopupInput(AiView, usize, Popup, Sender<String>),
}

/// LLM-backed Ai that implements the programmatic `Ai` trait by delegating to the async `AI` client
/// running inside a dedicated background thread (with its own Tokio runtime).
pub struct LlmAi {
    tx: Sender<LlmRequest>,
}

impl LlmAi {
    pub fn new(model: &'static str) -> Self {
        let (tx, rx): (Sender<LlmRequest>, Receiver<LlmRequest>) = mpsc::channel();

        // Spawn a background thread that owns a tokio runtime and the async LLM client
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
            let mut ai_client = AI::new(model);
            // Process requests
            while let Ok(req) = rx.recv() {
                match req {
                    LlmRequest::SelectAction(view, civ_idx, resp_tx) => {
                        // Build a simple prompt describing the view and possible actions
                        let mut prompt = format!("Turn: {}\nPlayer index: {}\nPlayers:\n", view.turn, civ_idx);
                        for (i, p) in view.players.iter().enumerate() {
                            prompt.push_str(&format!("- {}: resources={} buildings={} units={}\n", p.name, p.resources, p.buildings, p.units));
                        }
                        prompt.push_str("Possible buildings:\n");
                        for b in &view.buildings {
                            prompt.push_str(&format!("- {}\n", b));
                        }
                        prompt.push_str("Possible units:\n");
                        for u in &view.units {
                            prompt.push_str(&format!("- {}\n", u));
                        }
                        prompt.push_str("\nChoose one action (exactly as the action string, e.g. 'end' or 'build Farm' or 'hire Warrior' or 'attack playername'):\n");

                        let res = rt.block_on(ai_client.send_message(prompt));
                        // If model returns nothing, default to end
                        let out = res.or_else(|| Some("end".to_string()));
                        let _ = resp_tx.send(out);
                    }
                    LlmRequest::SelectPopupInput(view, civ_idx, popup, resp_tx) => {
                        // Build prompt describing popup
                        let mut prompt = format!("Popup for player {}: {}\nPrompt: {}\nChoices:\n", civ_idx, popup.title, popup.prompt);
                        for (i, c) in popup.choices.iter().enumerate() {
                            prompt.push_str(&format!("{}: {}\n", i+1, c));
                        }
                        prompt.push_str("Provide the input to select (either the 1-based index or the choice text):\n");
                        let res = rt.block_on(ai_client.send_message(prompt));
                        let chosen = res.unwrap_or_default();
                        let _ = resp_tx.send(chosen);
                    }
                }
            }
        });

        Self { tx }
    }
}

impl crate::game::Ai for LlmAi {
    fn select_action(&mut self, view: &AiView, civ_index: usize) -> Option<String> {
        let (resp_tx, resp_rx) = mpsc::channel();
        // Clone view to send across thread
        let view_cloned = AiView { turn: view.turn, player_turn: view.player_turn, players: view.players.clone(), buildings: view.buildings.clone(), units: view.units.clone(), seed: view.seed.clone() };
        if let Err(e) = self.tx.send(LlmRequest::SelectAction(view_cloned, civ_index, resp_tx)) {
            log::error!("Failed to send LLM select_action request: {}", e);
            return Some("end".to_string());
        }
        // Wait for response with a timeout
        match resp_rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(opt) => opt,
            Err(_) => {
                log::warn!("LLM select_action timed out for civ {}", civ_index);
                Some("end".to_string())
            }
        }
    }

    fn select_popup_input(&mut self, view: &AiView, civ_index: usize, popup: &Popup) -> String {
        let (resp_tx, resp_rx) = mpsc::channel();
        let view_cloned = AiView { turn: view.turn, player_turn: view.player_turn, players: view.players.clone(), buildings: view.buildings.clone(), units: view.units.clone(), seed: view.seed.clone() };
        if let Err(e) = self.tx.send(LlmRequest::SelectPopupInput(view_cloned, civ_index, popup.clone(), resp_tx)) {
            log::error!("Failed to send LLM select_popup_input request: {}", e);
            return String::new();
        }
        match resp_rx.recv_timeout(std::time::Duration::from_secs(10)) {
            Ok(s) => s,
            Err(_) => {
                log::warn!("LLM select_popup_input timed out for civ {}", civ_index);
                String::new()
            }
        }
    }
}