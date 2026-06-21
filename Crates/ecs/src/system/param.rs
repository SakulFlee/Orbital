use crate::system::access::ComponentAccess;

pub trait SystemParam: Sized {
    fn access() -> ComponentAccess;
}
