use super::err::BfResult;

pub trait SubCmd {
    fn run(self) -> BfResult<()>;
}
