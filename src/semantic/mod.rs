pub mod checker;
pub mod context;

#[cfg(test)]
mod tests;

pub use checker::check_program;