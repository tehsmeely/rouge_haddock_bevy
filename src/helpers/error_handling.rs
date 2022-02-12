use log::info;
pub trait ResultOkLog<T> {
    fn ok_log(self) -> Option<T>;
}
impl<T, E> ResultOkLog<T> for Result<T, E>
where
    E: std::error::Error,
{
    fn ok_log(self) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(e) => {
                info!("Called ok() on Error. Error: {}", e);
                None
            }
        }
    }
}
