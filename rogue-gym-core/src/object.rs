use path::ObjectPath;
pub trait Object {
    fn path(&self) -> ObjectPath;
}
