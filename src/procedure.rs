use console::{style, StyledObject, Term};

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
        Term::stdout().clear_last_lines(1).unwrap();
        println!("{} {}", self.get_step(), style(msg).green().bold());
    }

    pub fn next(&mut self, msg: String) {
        self.step += 1;
        println!("{} {}", self.get_step(), msg)
    }

    fn get_step(&self) -> StyledObject<String> {
        style(format!("[{}/{}]", self.step, self.max_steps)).dim()
    }   
}
