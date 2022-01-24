use anyhow::Result;

pub trait SubCmd {
    fn run(self) -> Result<()>;
}
