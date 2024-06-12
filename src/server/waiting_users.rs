use crate::common::SocketMessage;
use crate::server::User;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default, Clone)]
pub struct WaitingUsers(Arc<Mutex<Vec<User>>>);

impl WaitingUsers {
    pub async fn queue(&self, mut user: User) {
        let mut lock = self.0.lock().await;

        tracing::info!("queuing user: {}", &user.id);
        lock.push(user);
        tracing::info!("users waiting for peer: {}", lock.len());
    }

    pub async fn len(&self) -> usize {
        self.0.lock().await.len()
    }

    /// If 2 or more users are present, it'll pop the longest-waiting user along with
    /// another user who has the closest personality.
    pub async fn pop_pair(&self) -> Option<(User, User)> {
        let mut users = self.0.lock().await;

        let len = users.len();
        if len < 2 {
            return None;
        }

        // prioritize the user who waited the longest.
        let left = users.remove(0);

        let mut right_index = 0;
        let mut closest = f32::MAX;

        for (index, user) in users.iter().enumerate() {
            let diff = left.scores.distance(&user.scores);
            if diff < closest {
                closest = diff;
                right_index = index;
            }
        }

        let right = users.remove(right_index);

        tracing::info!("two users paired up! {} and {}", &left.id, &right.id);
        tracing::info!("remaining users: {}", users.len());

        Some((left, right))
    }
}
