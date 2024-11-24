#[derive(Default, Clone, Debug, PartialEq)]
pub struct Post {
    pub username: String,
    pub message: String,
    pub id: String,
    pub retweets: u32,
    pub likes: u32,
    pub time: String,
    pub avatar: String,
}