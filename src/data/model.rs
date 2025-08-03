#[derive(Default, Clone, Debug, PartialEq, Hash)]
pub struct Post {
    pub id: usize,
    pub username: String,
    pub message: String,
    pub time: String,
    pub avatar: String,
}
