pub struct State {}

impl fastn_package::Package {
    pub fn reader() -> fastn_section::ContinuationResult<State> {
        fastn_section::ContinuationResult::Stuck(Box::new(State {}), Default::default())
    }
}

impl fastn_section::Continuation for State {
    type Output = fastn_package::Package;
    type NeededInput = Vec<String>;
    // File name
    type NeededOutput = Vec<(String, Option<String>)>;

    fn continue_after(self, _n: Self::NeededOutput) -> fastn_section::ContinuationResult<Self> {
        todo!()
    }
}
