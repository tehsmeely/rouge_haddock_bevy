use code_location::CodeLocation;
use log::warn;

pub trait ResultOkLog<T> {
    fn ok_log(self, here: CodeLocation) -> Option<T>;
}
impl<T, E> ResultOkLog<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn ok_log(self, here: CodeLocation) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(e) => {
                warn!("Called ok() on Error. Error: {}, At {}", e, here);
                None
            }
        }
    }
}
