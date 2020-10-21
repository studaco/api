pub mod account;
pub mod lesson;
pub mod permission;
pub mod repeat;
pub mod teacher;

pub fn templated_insert(size: usize, iteration: usize) -> String {
    format!("({})", (0..size).into_iter().map(|i| format!("${}", iteration * size + i + 1)).collect::<Vec<String>>().join(", "))
}
