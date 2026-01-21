//! Thread lifecycle management.

use std::thread::{self, JoinHandle};

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Actor {
    Photographer,
    InputHandler,
    ShellForwarder,
}

impl Actor {
    pub fn name(self) -> &'static str {
        match self {
            Actor::Photographer => "photographer",
            Actor::InputHandler => "input-handler",
            Actor::ShellForwarder => "shell-forwarder",
        }
    }
}

pub struct Runtime {
    handles: Vec<(Actor, JoinHandle<Result<()>>)>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            handles: Vec::new(),
        }
    }

    pub fn spawn<F>(&mut self, actor: Actor, f: F)
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        let handle = thread::Builder::new()
            .name(actor.name().to_string())
            .spawn(f)
            .expect("Failed to spawn thread");
        self.handles.push((actor, handle));
    }

    pub fn join_all(self) -> Result<()> {
        let mut first_error: Option<anyhow::Error> = None;

        for (actor, handle) in self.handles {
            match handle.join() {
                Ok(Ok(())) => {
                    log::debug!("{:?} completed", actor);
                }
                Ok(Err(e)) => {
                    log::error!("{:?} failed: {}", actor, e);
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
                Err(_) => {
                    log::error!("{:?} panicked", actor);
                    if first_error.is_none() {
                        first_error = Some(anyhow::anyhow!("{:?} panicked", actor));
                    }
                }
            }
        }

        match first_error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
