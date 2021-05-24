use crate::util::BfResult;

pub trait SubCmd {
    fn run(self) -> BfResult<()>;
}
