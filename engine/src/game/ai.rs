use openai::chat::{
    ChatCompletionDelta, ChatCompletionMessage, ChatCompletionMessageRole,
};
use openai::Credentials;
use std::default::Default;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use crate::game::AiView;
use crate::game::state::Popup;

/// OpenAI-based AI client for the game.
///
/// This struct manages communication with an LLM API (OpenAI-compatible)
/// to make decisions for AI players in the game.
///
/// # Environment Variables
/// Requires the following environment variables:
/// - `OPENAI_KEY` or `OPENAI_API_KEY` - API authentication key
/// - `OPENAI_BASE_URL` or `OPENAI_API_BASE` - API endpoint URL (optional, defaults to OpenAI)
pub struct AI {
    credentials: Credentials,
    model: &'static str,
    messages: Vec<ChatCompletionMessage>,
}

impl AI {
    /// Create a new AI client with the specified model.
    ///
    /// # Arguments
    /// * `model` - The LLM model identifier (e.g., "openai/gpt-4o-mini")
    ///
    /// # Returns
    /// A new AI instance configured with system prompts for the game
    pub fn new(model: &'static str) -> Self {
        // Check if credentials are set
        let has_key = std::env::var("OPENAI_KEY").or_else(|_| std::env::var("OPENAI_API_KEY")).is_ok();
        let has_url = std::env::var("OPENAI_BASE_URL").or_else(|_| std::env::var("OPENAI_API_BASE")).is_ok();

        if !has_key {
            log::warn!("OPENAI_KEY or OPENAI_API_KEY environment variable not set - AI will not work properly");
        }
        if !has_url {
            log::warn!("OPENAI_BASE_URL or OPENAI_API_BASE environment variable not set - using default OpenAI endpoint");
        }

        AI {
            credentials: Credentials::from_env(),
            model,
            messages: vec![ChatCompletionMessage {
                role: ChatCompletionMessageRole::System,
                content: Some("You are an AI playing a civilization-like game in a text interface.\
                \nRules:\
                \n- This is a turn-based game between human and AI players.\
                \n- Each player has one city.\
                \n- To win, you must destroy your opponent's city.\
                \n- Each turn you can perform MULTIPLE actions (build, hire, attack).\
                \n- When you're done with your actions, you MUST say 'end' to finish your turn.\
                \n- I will ask you for one action at a time. After each action succeeds, I'll ask again.\
                \n- Keep doing actions until you feel ready, then say 'end'.\
                \n- IMPORTANT: You should say 'end' after 2-4 actions to give your opponent a chance.\
                \n\nIMPORTANT: Your response must ONLY be the action string, nothing else. Do NOT use JSON, do NOT use code blocks, do NOT add explanations.\
                \nValid action formats:\
                \n- 'end' (to end your turn - USE THIS after a few actions)\
                \n- 'build <building_name>' (e.g., 'build farm')\
                \n- 'hire <unit_name>' (e.g., 'hire warrior')\
                \n- 'attack <player_name>' (e.g., 'attack player1')\
                \n\nRespond with ONLY the action text, exactly as shown above.".to_string()),
                ..Default::default()
            }],
        }
    }

    /// Send a message to the LLM and return the content (if any).
    ///
    /// This method sends a user message to the LLM, waits for a response,
    /// and returns the text content. The conversation history is maintained
    /// in the messages vector.
    ///
    /// # Arguments
    /// * `text` - The message to send to the LLM
    ///
    /// # Returns
    /// Some(String) with the LLM's response, or None if the request fails
    /// or no response is received
    pub async fn send_message(&mut self, text: String) -> Option<String> {
        let message = ChatCompletionMessage {
            role: ChatCompletionMessageRole::User,
            content: Some(text),
            ..Default::default()
        };

        self.messages.push(message);

        log::debug!("Sending {} messages to LLM (model={})", self.messages.len(), self.model);
        let chat_completion_res = ChatCompletionDelta::builder(self.model, self.messages.clone())
            .credentials(self.credentials.clone())
            .create()
            .await;

        let chat_completion = match chat_completion_res {
            Ok(c) => c,
            Err(e) => {
                log::error!("AI chat completion failed: {e}");
                return None;
            }
        };

        log::debug!("Chat completion received with {} choices", chat_completion.choices.len());
        let returned_message_opt = chat_completion.choices.first().map(|c| c.message.clone());
        let Some(returned_message) = returned_message_opt else {
                log::warn!("AI chat completion returned no choices (model={}, messages={})", self.model, self.messages.len());
                return None;
            };

        self.messages.push(returned_message.clone());

        let content = returned_message.content.as_deref().map(|s| s.trim().to_string());
        log::debug!("AI response ({:?}): {}", returned_message.role, content.as_deref().unwrap_or(""));
        content
    }
}

// ===== LLM-backed Ai adapter =====

/// Internal request types for communication with the LLM background thread.
enum LlmRequest {
    SelectAction(AiView, usize, Sender<Option<String>>),
    SelectPopupInput(AiView, usize, Popup, Sender<String>),
}

/// LLM-backed AI implementation that bridges the synchronous Ai trait
/// to the async OpenAI client.
///
/// This struct spawns a background thread with a Tokio runtime to handle
/// async LLM requests while presenting a synchronous interface.
pub struct LlmAi {
    tx: Sender<LlmRequest>,
}

impl LlmAi {
    /// Clean LLM response by stripping JSON formatting, code blocks, and other noise.
    ///
    /// The LLM sometimes responds with formatted output like JSON or markdown code blocks,
    /// even when instructed not to. This function extracts the actual action string.
    ///
    /// # Arguments
    /// * `response` - The raw LLM response text
    ///
    /// # Returns
    /// The cleaned action string
    fn clean_llm_response(response: &str) -> String {
        let mut cleaned = response.trim().to_string();

        // Remove markdown code blocks (```json, ```, etc.)
        if cleaned.starts_with("```") {
            // Find first newline after opening ```
            if let Some(first_newline) = cleaned.find('\n') {
                cleaned = cleaned[first_newline + 1..].to_string();
            }
            // Remove closing ```
            if let Some(last_backticks) = cleaned.rfind("```") {
                cleaned = cleaned[..last_backticks].to_string();
            }
        }

        cleaned = cleaned.trim().to_string();

        // Try to parse as JSON and extract "action" field
        if cleaned.starts_with('{') && cleaned.ends_with('}') {
            // Simple JSON parsing for {"action": "..."} format
            if let Some(action_start) = cleaned.find("\"action\"") {
                let after_action = &cleaned[action_start + 8..]; // Skip "action"
                if let Some(colon_pos) = after_action.find(':') {
                    let after_colon = &after_action[colon_pos + 1..];
                    if let Some(quote_start) = after_colon.find('"') {
                        let after_quote = &after_colon[quote_start + 1..];
                        if let Some(quote_end) = after_quote.find('"') {
                            return after_quote[..quote_end].trim().to_string();
                        }
                    }
                }
            }
        }

        // Return cleaned response
        cleaned
    }

    /// Create a new LLM-backed AI instance.
    ///
    /// This spawns a background thread with a Tokio runtime that processes
    /// AI requests asynchronously. The thread will run until the LlmAi
    /// instance is dropped and the channel is closed.
    ///
    /// # Arguments
    /// * `model` - The LLM model identifier (must be a static string)
    ///
    /// # Returns
    /// A new LlmAi instance ready to handle AI requests
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
                        let mut prompt = format!("=== TURN {} ===\nYou are player: {}\n\nPlayers:\n", view.turn, view.players.get(civ_idx).map(|p| &p.name).unwrap_or(&"Unknown".to_string()));
                        for (i, p) in view.players.iter().enumerate() {
                            let marker = if i == civ_idx { " <- YOU" } else { "" };
                            prompt.push_str(&format!("  {} - Resources: {}, Buildings: {}, Units: {}{}\n",
                                p.name, p.resources, p.buildings, p.units, marker));
                        }
                        prompt.push_str("\nAvailable buildings to build:\n");
                        for b in &view.buildings {
                            prompt.push_str(&format!("  - {}\n", b.to_lowercase()));
                        }
                        prompt.push_str("\nAvailable units to hire:\n");
                        for u in &view.units {
                            prompt.push_str(&format!("  - {}\n", u.to_lowercase()));
                        }
                        prompt.push_str("\nYour action (respond with ONLY ONE of these, nothing else):\n");
                        prompt.push_str("  end\n");
                        for b in &view.buildings {
                            prompt.push_str(&format!("  build {}\n", b.to_lowercase()));
                        }
                        for u in &view.units {
                            prompt.push_str(&format!("  hire {}\n", u.to_lowercase()));
                        }
                        for (i, p) in view.players.iter().enumerate() {
                            if i != civ_idx {
                                prompt.push_str(&format!("  attack {}\n", p.name.to_lowercase()));
                            }
                        }

                        let res = rt.block_on(ai_client.send_message(prompt));
                        // Parse and clean the response
                        let out = res.map(|s| Self::clean_llm_response(&s)).or_else(|| Some("end".to_string()));
                        let _ = resp_tx.send(out);
                    }
                    LlmRequest::SelectPopupInput(_view, civ_idx, popup, resp_tx) => {
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
            log::error!("Failed to send LLM select_action request: {e}");
            return Some("end".to_string());
        }
        // Wait for response with a timeout
        if let Ok(opt) = resp_rx.recv_timeout(std::time::Duration::from_secs(10)) { opt } else {
            log::warn!("LLM select_action timed out for civ {civ_index}");
            Some("end".to_string())
        }
    }

    fn select_popup_input(&mut self, view: &AiView, civ_index: usize, popup: &Popup) -> String {
        let (resp_tx, resp_rx) = mpsc::channel();
        let view_cloned = AiView { turn: view.turn, player_turn: view.player_turn, players: view.players.clone(), buildings: view.buildings.clone(), units: view.units.clone(), seed: view.seed.clone() };
        if let Err(e) = self.tx.send(LlmRequest::SelectPopupInput(view_cloned, civ_index, popup.clone(), resp_tx)) {
            log::error!("Failed to send LLM select_popup_input request: {e}");
            return String::new();
        }
        if let Ok(s) = resp_rx.recv_timeout(std::time::Duration::from_secs(10)) { s } else {
            log::warn!("LLM select_popup_input timed out for civ {civ_index}");
            String::new()
        }
    }
}