
// pub struct Storage;

// impl Storage {
//     async fn set_global_model(
//         &mut self,
//         round_id: u64,
//         round_seed: &RoundSeed,
//         global_model: &Model,
//     ) -> StorageResult<String> {
//         self.model
//             .set_global_model(round_id, round_seed, global_model)
//             .await
//     }

//     async fn global_model(&mut self, id: &str) -> StorageResult<Option<Model>> {
//         self.model.global_model(id).await
//     }
// }