/// In the game, we identify all objects by 'path', for dara driven architecture
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct ObjectPath {
    inner: PathImpl,
}

/// commonly our dungeon has hierarchy, so
/// use path to specify 'where' is good(though I'm not sure)
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PlacePath {
    inner: PathImpl,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct PathImpl(Vec<String>);

/// Object identifier
pub trait Path: Sized {
    fn as_path(&self) -> &PathImpl;
    fn as_path_mut(&mut self) -> &mut PathImpl;
    fn from_path(p: PathImpl) -> Self;
    /// construct single path from string
    fn from_str<S: AsRef<str>>(s: S) -> Self {
        let s = s.as_ref().to_owned();
        Self::from_path(PathImpl(vec![s]))
    }
    /// take 'string' and make self 'path::another_path::string'
    fn push<S: AsRef<str>>(&mut self, s: S) {
        let s = s.as_ref().to_owned();
        self.as_path_mut().0.push(s)
    }
    /// concat 2 paths
    fn append(&mut self, mut other: Self) {
        let other = &mut other.as_path_mut().0;
        self.as_path_mut().0.append(other);
    }
}

macro_rules! impl_path {
    () => {
        fn as_path(&self) -> &PathImpl {
            &self.inner
        }
        fn as_path_mut(&mut self) -> &mut PathImpl {
            &mut self.inner
        }
        fn from_path(p: PathImpl) -> Self {
            Self { inner: p }
        }
    }
}

impl Path for ObjectPath {
    impl_path!();
}

impl Path for PlacePath {
    impl_path!();
}
