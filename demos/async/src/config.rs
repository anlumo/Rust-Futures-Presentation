use serde_derive::Deserialize;

#[derive(Debug,Deserialize)]
pub struct Job(pub String);

pub fn load_config<T: std::io::Read>(reader: T) -> Result<Vec<Job>, serde_json::Error> {
    serde_json::from_reader(reader)
}
