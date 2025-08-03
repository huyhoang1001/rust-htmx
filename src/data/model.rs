#[derive(Default, Clone, Debug, PartialEq, Hash)]
pub struct Post {
    pub id: u64,
    pub username: String,
    pub message: String,
    pub time: String,
    pub avatar: String,
    pub owner_id: String, // Simple string-based owner identification
}
