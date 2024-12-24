pub const CLEAR: &str = "\x1B[2J\x1B[1;1H";
pub const RESET: &str = "\x1B[0m";
pub const WHITE: &str = "\x1B[97m";
pub const BLUE: &str = "\x1B[34m";
pub const CYAN: &str = "\x1B[36m";
pub const GRAY: &str = "\x1B[37m";
pub const RED: &str = "\x1B[31m";
/*
Regular Files: White (\x1B[37m)
Directories: Blue (\x1B[34m)
Symbolic Links: Cyan (\x1B[36m)
Executable Files: Green (\x1B[32m)
Archive Files: Magenta (\x1B[35m)
Compressed Files: Yellow (\x1B[33m)
Socket Files: Magenta (\x1B[35m)
FIFO Files: Yellow (\x1B[33m)
Device Files: Red (\x1B[31m)
 */
