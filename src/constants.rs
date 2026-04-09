pub const EXIT_CMD: &str = "exit";
pub const ECHO_CMD: &str = "echo";
pub const TYPE_CMD: &str = "type";
pub const PWD_CMD: &str = "pwd";
pub const CD_CMD: &str = "cd";
pub const HISTORY_CMD: &str = "history";
pub const JOBS_CMD: &str = "jobs";
pub const HOME_DIR: &str = "~";

pub const PROMPT: &str = "$ ";

// TODO: Improve. This requires that each new built-in command shall be added manually.
pub const SHELL_BUILTINS: &[&str] = &[EXIT_CMD, 
                                      ECHO_CMD, 
                                      TYPE_CMD, 
                                      PWD_CMD, 
                                      CD_CMD, 
                                      HISTORY_CMD,
                                      JOBS_CMD];