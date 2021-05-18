use std::error::Error;

pub trait SubCmd {
    fn run(self) -> Result<(), Box<dyn Error>>;
}
