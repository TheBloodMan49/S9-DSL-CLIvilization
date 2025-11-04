// Include generated sources
include!(concat!(env!("OUT_DIR"), "/ast.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    const sample: &str = include_str!("../../examples/variant_1/game.json");

    #[test]
    fn test_sample() {
        println!(
            "{:?}",
            match serde_json::from_str::<Model>(sample) {
                Ok(ast) => ast,
                Err(e) => panic!("{}", e),
            }
        );
    }
}
