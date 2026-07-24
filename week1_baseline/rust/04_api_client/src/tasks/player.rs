use super::base::Task;

pub struct Player;

impl Task for Player {
    fn task_name() -> &'static str {
        "player"
    }
}
