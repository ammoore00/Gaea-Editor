use std::error::Error;
use std::fmt::Debug;
use std::result;
use std::sync::{Arc, Mutex};
use dashmap::DashMap;
use tokio::sync::RwLock;
use crate::data::domain::project::ProjectID;
use crate::data::domain::resource::resource::ResourceID;

pub trait UndoProvider {
    fn execute(&self, command_data: CommandData) -> Result<()>;
    fn undo(&self, undo_stack_id: UndoStackID) -> Result<()>;
}

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
}

impl Default for UndoService {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoProvider for UndoService {
    fn execute(&self, command_data: CommandData) -> Result<()> {
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

    fn undo(&self, undo_stack_id: UndoStackID) -> Result<()> {
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

#[derive(Clone)]
pub enum CommandData {
    Single {
        command: Arc<dyn Command>,
        undo_stack_id: UndoStackID,
    },
    Global {
        command: Arc<dyn GlobalCommand>,
    },
}

pub trait Command: Sync + Send {
    fn execute(&self) -> result::Result<(), anyhow::Error>;
    fn undo(&self) -> result::Result<(), anyhow::Error>;
}

pub trait GlobalCommand: Command {
    fn affected_resources(&self) -> &Vec<UndoStackID>;
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    struct TestCommand {
        execute_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
        undo_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
    }
    
    impl TestCommand {
        fn new(
            execute_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
            undo_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
        ) -> Self {
            Self {
                execute_fn,
                undo_fn,
            }
        }
    }
    
    impl Command for TestCommand {
        fn execute(&self) -> result::Result<(), anyhow::Error> {
            (self.execute_fn)(&self)
        }

        fn undo(&self) -> result::Result<(), anyhow::Error> {
            (self.undo_fn)(&self)
        }
    }

    struct TestGlobalCommand {
        execute_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
        undo_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
        affected_resources: Vec<UndoStackID>,
    }

    impl TestGlobalCommand {
        fn new(
            execute_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
            undo_fn: Box<dyn Fn(&Self) -> result::Result<(), anyhow::Error> + Send + Sync>,
            affected_resources: Vec<UndoStackID>,
        ) -> Self {
            Self {
                execute_fn,
                undo_fn,
                affected_resources,
            }
        }
    }
    
    impl Command for TestGlobalCommand {
        fn execute(&self) -> result::Result<(), anyhow::Error> {
            (self.execute_fn)(&self)
        }

        fn undo(&self) -> result::Result<(), anyhow::Error> {
            (self.undo_fn)(&self)
        }
    }

    impl GlobalCommand for TestGlobalCommand {
        fn affected_resources(&self) -> &Vec<UndoStackID> {
            &self.affected_resources
        }
    }
    
    #[derive(Default)]
    struct CallTracker {
        execute_count: Mutex<usize>,
        undo_count: Mutex<usize>,
    }

    /// Test standard command execution and reversion
    #[test]
    fn test_commands() {
        let undo_service = UndoService::new();

        let call_tracker = Arc::new(CallTracker::default());
        let execute_tracker = Arc::clone(&call_tracker);
        let undo_tracker = Arc::clone(&call_tracker);

        let data = Arc::new(Mutex::new(0));
        let execute_data = Arc::clone(&data);
        let undo_data = Arc::clone(&data);

        // Given a command which increments and decrements a number

        let command = TestCommand::new(
            Box::new(move |_| {
                {
                    let mut count = execute_tracker.execute_count.lock().unwrap();
                    *count += 1;
                }
                {
                    let mut data = execute_data.lock().unwrap();
                    *data += 1;
                }
                Ok(())
            }),
            Box::new(move |_| {
                {
                    let mut count = undo_tracker.undo_count.lock().unwrap();
                    *count += 1;
                }
                {
                    let mut data = undo_data.lock().unwrap();
                    *data -= 1;
                }
                Ok(())
            }),
        );

        let undo_stack_id = UndoStackID::Resource(Uuid::default());
        let command_data = CommandData::Single {
            command: Arc::new(command),
            undo_stack_id: undo_stack_id.clone(),
        };

        // When I execute that command

        undo_service.execute(command_data.clone()).unwrap();
        undo_service.execute(command_data.clone()).unwrap();
        undo_service.execute(command_data.clone()).unwrap();
        undo_service.execute(command_data.clone()).unwrap();

        // Then the right number of calls and the right data should come out

        assert_eq!(*call_tracker.execute_count.lock().unwrap(), 4);
        assert_eq!(*data.lock().unwrap(), 4);

        // When I undo that command

        undo_service.undo(undo_stack_id).unwrap();

        // Then the data should be decremented, and call counters incremented as appropriate

        assert_eq!(*call_tracker.execute_count.lock().unwrap(), 4);
        assert_eq!(*call_tracker.undo_count.lock().unwrap(), 1);
        assert_eq!(*data.lock().unwrap(), 3);
    }

    /// Test global command execution and reversion
    #[test]
    fn test_global_commands() {
        let undo_service = UndoService::new();

        let call_tracker = Arc::new(CallTracker::default());
        let execute_tracker = Arc::clone(&call_tracker);
        let undo_tracker = Arc::clone(&call_tracker);

        let data = Arc::new(Mutex::new(0));
        let execute_data = Arc::clone(&data);
        let undo_data = Arc::clone(&data);

        // Given a global command which increments and decrements a number

        let command = TestGlobalCommand::new(
            Box::new(move |_| {
                {
                    let mut count = execute_tracker.execute_count.lock().unwrap();
                    *count += 1;
                }
                {
                    let mut data = execute_data.lock().unwrap();
                    *data += 1;
                }
                Ok(())
            }),
            Box::new(move |_| {
                {
                    let mut count = undo_tracker.undo_count.lock().unwrap();
                    *count += 1;
                }
                {
                    let mut data = undo_data.lock().unwrap();
                    *data -= 1;
                }
                Ok(())
            }),
            Vec::new(),
        );

        let undo_stack_id = UndoStackID::Global;
        let command_data = CommandData::Global {
            command: Arc::new(command),
        };

        // When I execute that command

        undo_service.execute(command_data.clone()).unwrap();
        undo_service.execute(command_data.clone()).unwrap();
        undo_service.execute(command_data.clone()).unwrap();
        undo_service.execute(command_data.clone()).unwrap();

        // Then the right number of calls and the right data should come out

        assert_eq!(*call_tracker.execute_count.lock().unwrap(), 4);
        assert_eq!(*data.lock().unwrap(), 4);

        // When I undo that command

        undo_service.undo(undo_stack_id).unwrap();

        // Then the data should be decremented, and call counters incremented as appropriate

        assert_eq!(*call_tracker.execute_count.lock().unwrap(), 4);
        assert_eq!(*call_tracker.undo_count.lock().unwrap(), 1);
        assert_eq!(*data.lock().unwrap(), 3);
    }

    /// Test multiple standard commands running concurrently accessing different resources
    #[test]
    fn test_concurrent_commands() {}

    /// Test multiple standard commands running concurrently accessing the same resource,
    /// which should lock the resource and force sequential execution
    #[test]
    fn test_concurrent_commands_shared_resource() {}

    /// Test that multiple global commands will lock and run sequentially
    #[test]
    fn test_concurrent_global_commands() {}

    /// Test that global commands lock standard commands until they are done
    #[test]
    fn test_concurrent_commands_mixed() {}
}