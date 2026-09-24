
#[derive(Debug)]
pub enum ActionResult {
    Success,

    Screenshot(Vec<u8>),

    UiDump(String)
}