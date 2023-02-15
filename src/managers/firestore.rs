use super::super::settings::Settings;
use firestore::*;

pub struct FirestoreManager {
    pub client: FirestoreDb,
    #[allow(dead_code)]
    key_file: String,
}

impl FirestoreManager {
    pub async fn new() -> Self {
        let settings = Settings::new();
        // std::env::set_var(
        //     "GOOGLE_APPLICATION_CREDENTIALS",
        //     &settings.firestore.key_filename,
        // );
        let client = FirestoreDb::new(&settings.firestore.project_id)
            .await
            .expect("Failed to create Firestore client");
        Self {
            client,
            key_file: settings.firestore.key_filename.clone(),
        }
    }
}
