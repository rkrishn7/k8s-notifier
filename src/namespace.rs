#[derive(Debug, Clone)]
pub enum NamespaceScope {
    All,
    Names(Vec<String>),
}
