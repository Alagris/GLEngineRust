use ocl::core::Error;
use std::fmt::{Display, Formatter, Debug};

#[derive(Debug, Fail)]
pub struct ClGlError{
    err:Error
}
impl From<Error> for ClGlError{
    fn from(err: Error) -> Self {
        Self{err}
    }
}
impl Display for ClGlError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.err,f)
    }
}