use rand::Rng; 
use rand::distributions::Alphanumeric;

pub fn random_token(len: usize) -> String {
    rand::thread_rng()
    .sample_iter(&Alphanumeric)
    .take(len)
    .collect::<String>()
}    