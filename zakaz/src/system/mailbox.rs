use std::sync::Arc;
use chrono::Local;
use mailbox_processor::{BufferSize, MailboxProcessor};
use tokio::sync::Mutex;

use crate::{
    inf, err, notify_channel,
    system::{
        state::State,
        types::{RuntimeInMessage, RuntimeOutMessage, UIMessage},
    },
};

pub struct Mailbox;

impl Mailbox {
    #[allow(dead_code)]
    fn save_state(state: &State) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        inf!("Saving state.");
        match state.save() {
            Ok(_) => Ok(()),
            Err(e) => {
                err!("Error saving state: {:?}", e);
                Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
            }
        }
    }

    pub async fn make() -> Arc<Mutex<MailboxProcessor<RuntimeInMessage<State>, RuntimeOutMessage<State>>>> {
        let mb = MailboxProcessor::<RuntimeInMessage<State>, RuntimeOutMessage<State>>::new(
            BufferSize::Default,
            State::new(),
            |msg, state, reply_channel| async move {
                match msg {
                    RuntimeInMessage::NewState(new_state) => {
                        inf!("Setting new state.");
                        notify_channel!(reply_channel, RuntimeOutMessage::Ok);
                        new_state
                    }

                    RuntimeInMessage::Start => {
                        inf!("Starting runtime.");
                        let mut state_local = state.clone();
                        state_local.version += 1;
                        state_local.start_time = Local::now();
                        state_local.is_running = true;

                        // Notify UI that runtime started
                        state.send_message_to_ui(UIMessage::RuntimeStarted);
                        state.send_message_to_ui(UIMessage::StatusMessage("Runtime started successfully".to_string()));
                        
                        let out_msg = RuntimeOutMessage::Started(state_local.start_time);
                        notify_channel!(reply_channel, out_msg);
                        state_local
                    }

                    RuntimeInMessage::Stop => {
                        inf!("Stopping runtime.");
                        let mut state_local = state.clone();
                        state_local.version += 1;
                        state_local.is_running = false;

                        // Notify UI that runtime stopped
                        state.send_message_to_ui(UIMessage::RuntimeStopped);
                        state.send_message_to_ui(UIMessage::StatusMessage("Runtime stopped".to_string()));
                        
                        notify_channel!(reply_channel, RuntimeOutMessage::Ok);
                        state_local
                    }

                    RuntimeInMessage::State => {
                        inf!("Getting state.");
                        let time = chrono::Utc::now();
                        let state_clone = state.clone();
                        notify_channel!(reply_channel, RuntimeOutMessage::State(state_clone, time));
                        state
                    }

                    RuntimeInMessage::IncrementCounter => {
                        inf!("Incrementing counter.");
                        let mut state_local = state.clone();
                        state_local.version += 1;
                        state_local.counter += 1;

                        // Notify UI of counter change
                        state.send_message_to_ui(UIMessage::UpdateCounter(state_local.counter));
                        
                        notify_channel!(reply_channel, RuntimeOutMessage::Ok);
                        state_local
                    }

                    RuntimeInMessage::DecrementCounter => {
                        inf!("Decrementing counter.");
                        let mut state_local = state.clone();
                        state_local.version += 1;
                        state_local.counter -= 1;

                        // Notify UI of counter change
                        state.send_message_to_ui(UIMessage::UpdateCounter(state_local.counter));
                        
                        notify_channel!(reply_channel, RuntimeOutMessage::Ok);
                        state_local
                    }

                    RuntimeInMessage::ResetCounter => {
                        inf!("Resetting counter.");
                        let mut state_local = state.clone();
                        state_local.version += 1;
                        state_local.counter = 0;

                        // Notify UI of counter change
                        state.send_message_to_ui(UIMessage::UpdateCounter(state_local.counter));
                        state.send_message_to_ui(UIMessage::StatusMessage("Counter reset to zero".to_string()));
                        
                        notify_channel!(reply_channel, RuntimeOutMessage::Ok);
                        state_local
                    }

                    RuntimeInMessage::Error(error_msg) => {
                        err!("Error received: {}", error_msg);
                        state.send_message_to_ui(UIMessage::ErrorMessage(error_msg.clone()));
                        notify_channel!(reply_channel, RuntimeOutMessage::Error(error_msg));
                        state
                    }
                }
            }
        ).await;

        Arc::new(Mutex::new(mb))
    }
}