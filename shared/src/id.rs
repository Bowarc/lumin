// max id 18_446_744_073_709_551_615
static A_RLY_GOOD_VARIABLE_NAME: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Clone)]
pub struct ID(usize);

impl Default for ID {
    fn default() -> Self {
        let new_id = A_RLY_GOOD_VARIABLE_NAME.load(std::sync::atomic::Ordering::Relaxed);
        A_RLY_GOOD_VARIABLE_NAME.store(new_id + 1, std::sync::atomic::Ordering::Relaxed);
        ID(new_id)
    }
}

impl ID {
    pub fn new() -> Self {
        Self::default()
    }
}
