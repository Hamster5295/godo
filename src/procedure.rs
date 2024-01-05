use console::{style, StyledObject};

pub struct Procedure {
    max_steps: i32,
    step: i32,
}

pub fn new(steps: i32) -> Procedure {
    Procedure {
        max_steps: steps,
        step: 0,
    }
}

impl Procedure {
    pub fn finish(&self, msg: String) {
        print!("{} {}", self.get_step(), msg);
    }

    pub fn next(&mut self, msg: String) {
        self.step += 1;
        println!("{} {}", self.get_step(), style(msg).green().bold())
    }

    fn get_step(&self) -> StyledObject<std::string::String> {
        style(format!("[{}/{}]", self.step, self.max_steps)).dim()
    }
}
