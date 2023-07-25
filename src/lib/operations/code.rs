use rand::prelude::*;

pub fn generate_code() -> String {
    let code: Vec<String> = (0..6)
        .map(|_| rand::thread_rng().gen_range(0..10).to_string())
        .collect();

    code.join("")
}

#[cfg(test)]
mod tests {
    use crate::operations::code::generate_code;

    #[test]
    fn generate_code_test() {
        let code = generate_code();
        assert_eq!(code.len(), 6);
    }
}
