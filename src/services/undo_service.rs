use std::error::Error;
use std::result;
use std::sync::{Mutex};
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::domain::project::ProjectID;
use crate::domain::resource::resource::ResourceID;

pub struct UndoService {
    undo_stack: DashMap<UndoStackID, Mutex<Vec<CommandData>>>,
    global_lock: RwLock<()>,
}

// TODO: Implement some sort of dependency tracking and unified command queue - definitely beyond MVP though
// TODO: Add some sort of retry logic to undoing commands
impl UndoService {
    pub fn new() -> Self {
        Self {
            undo_stack: DashMap::new(),
            global_lock: RwLock::new(()),
        }
    }

    pub fn execute(&self, command_data: CommandData) -> Result<()> {
        match &command_data {
            CommandData::Global { command } => {
                // Obtain the global write lock, stopping all other operations
                let _global_lock = self.global_lock.write();

                // Dash map is thread safe for access
                let command_stack_mutex = self.undo_stack.entry(UndoStackID::Global).or_insert_with(|| Mutex::new(Vec::new()));

                // Still lock the resource to be able to read it, even though we have the global lock
                let mut command_stack = command_stack_mutex.lock().unwrap();
                command.execute().map_err(UndoError::CommandError)?;
                command_stack.push(command_data);

                Ok(())
            }
            CommandData::Single { command, undo_stack_id } => {
                // Get a read lock on global lock
                // This prevents single file operations from happening while a global operation is happening
                // but otherwise lets as many single operations happen in parallel as we want, as long as
                // they are not on the same resource
                let _global_lock = self.global_lock.read();

                // Dash map is thread safe for access
                let command_stack_mutex = self.undo_stack.entry(undo_stack_id.clone()).or_insert_with(|| Mutex::new(Vec::new()));

                // Command stack lock for a single resource to protect critical section
                let mut command_stack = command_stack_mutex.lock().unwrap();
                command.execute().map_err(UndoError::CommandError)?;
                command_stack.push(command_data);

                Ok(())
            }
        }
    }

    pub fn undo(&mut self, undo_stack_id: UndoStackID) -> Result<()> {
        match &undo_stack_id {
            UndoStackID::Global => {
                // Obtain the global write lock, stopping all other operations
                let _global_lock = self.global_lock.write();

                // Dash map is thread safe for access
                let command_stack_mutex = self.undo_stack.get_mut(&UndoStackID::Global).ok_or(UndoError::InvalidStackID(undo_stack_id))?;

                let mut command_stack = command_stack_mutex.lock().unwrap();
                let command_data = command_stack.last().ok_or(UndoError::NothingToUndo)?;

                match command_data {
                    CommandData::Single { command, .. } => {
                        command.undo().map_err(UndoError::CommandError)?;
                    }
                    CommandData::Global { command } => {
                        command.undo().map_err(UndoError::CommandError)?;
                    }
                }

                Ok(())
            }
            _ => {
                // Get a read lock on global lock
                // This prevents single file operations from happening while a global operation is happening
                // but otherwise lets as many single operations happen in parallel as we want, as long as
                // they are not on the same resource
                let _global_lock = self.global_lock.read();

                // Dash map is thread safe for access
                let command_stack_mutex = self.undo_stack.get_mut(&undo_stack_id).ok_or(UndoError::InvalidStackID(undo_stack_id))?;

                let mut command_stack = command_stack_mutex.lock().unwrap();
                let command_data = command_stack.last().ok_or(UndoError::NothingToUndo)?;

                match command_data {
                    CommandData::Single { command, .. } => {
                        command.undo().map_err(UndoError::CommandError)?;
                    }
                    CommandData::Global { command } => {
                        command.undo().map_err(UndoError::CommandError)?;
                    }
                }

                Ok(())
            }
        }
    }
}

pub type Result<T> = result::Result<T, UndoError>;

#[derive(Debug, thiserror::Error)]
pub enum UndoError {
    #[error("Nothing to undo!")]
    NothingToUndo,
    #[error("Invalid Stack ID {0:?}!")]
    InvalidStackID(UndoStackID),
    #[error(transparent)]
    CommandError(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum UndoStackID {
    Project(ProjectID),
    Resource(ResourceID),
    Global,
}

pub enum CommandData {
    Single {
        command: Box<dyn Command>,
        undo_stack_id: UndoStackID,
    },
    Global {
        command: Box<dyn GlobalCommand>,
    },
}

pub trait Command: Sync + Send {
    fn execute(&self) -> result::Result<(), anyhow::Error>;
    fn undo(&self) -> result::Result<(), anyhow::Error>;
}

pub trait GlobalCommand: Command {
    fn affected_resources(&self) -> Vec<UndoStackID>;
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestCommand;
    impl Command for TestCommand {
        fn execute(&self) -> result::Result<(), anyhow::Error> {
            todo!()
        }

        fn undo(&self) -> result::Result<(), anyhow::Error> {
            todo!()
        }
    }

    struct TestGlobalCommand;
    impl Command for TestGlobalCommand {
        fn execute(&self) -> result::Result<(), anyhow::Error> {
            todo!()
        }

        fn undo(&self) -> result::Result<(), anyhow::Error> {
            todo!()
        }
    }

    impl GlobalCommand for TestGlobalCommand {
        fn affected_resources(&self) -> Vec<UndoStackID> {
            todo!()
        }
    }
    
    mod execute {
        use super::*;
    }
    
    mod undo {
        use super::*;
    }
}