use crate::server::User;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Default, Clone, Debug)]
pub struct WaitingUsers(pub Arc<Mutex<Vec<User>>>);

impl WaitingUsers {
    #[cfg(test)]
    pub async fn user_ids(&self) -> Vec<String> {
        self.0
            .lock()
            .await
            .iter()
            .map(|user| user.id.to_owned())
            .collect()
    }

    pub async fn queue(&self, user: User) {
        tracing::info!("{}: queue i guess", &user.id);
        let mut lock = self.0.lock().await;

        let pos = lock.iter().position(|waiters| waiters.id == user.id);
        if let Some(pos) = pos {
            lock[pos].close();
            tracing::error!("User already in queue: {}", &user.id);
        }

        drop(lock);

        for _ in 0..10 {
            {
                let mut lock = self.0.lock().await;
                if lock
                    .iter()
                    .position(|waiters| waiters.id == user.id)
                    .is_none()
                {
                    lock.push(user);
                    tracing::info!("users waiting for peer: {}", lock.len());
                    return;
                } else {
                    tracing::info!("still there!");
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        tracing::error!("{}: failed to insert user.", &user.id);
    }

    pub async fn take(&self, id: &str) -> Option<User> {
        let pos = {
            let lock = self.0.lock().await;

            lock.iter().position(|user| user.id == id)?
        };
        let user = self.0.lock().await.remove(pos);

        Some(user)
    }

    pub async fn len(&self) -> usize {
        self.0.lock().await.len()
    }

    pub async fn contains(&self, id: &str) -> bool {
        self.0
            .lock()
            .await
            .iter()
            .position(|x| &x.id == id)
            .is_some()
    }

    /// If 2 or more users are present, it'll pop the longest-waiting user along with
    /// another user who has the closest personality.
    pub async fn pop_pair(&self) -> Option<(User, User)> {
        tracing::info!("pop pair called!");
        let (left, right) = {
            let mut users = self.0.lock().await;
            tracing::info!("lock acquired! :D");

            users.retain_mut(|user| !user.is_closed());

            let len = users.len();
            if len < 2 {
                tracing::info!("not enough users to for pairing. qty: {}", len);
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
            (left, right)
        };

        Some((left, right))
    }
}
