use agent::Agent;

mod test {
    use super::*;
    use agent::models::llm::candle::Candle;
    use agent::models::llm::token_output_stream::TokenOutputStream;
    use candle::{Result, Tensor};
    use std::sync::Arc;

    #[test]
    fn test_token_output_stream() -> Result<()> {
        let tokenizer = tokenizers::Tokenizer::from_file("path/to/tokenizer.json")?;
        let mut stream = TokenOutputStream::new(tokenizer);

        // Simulate token generation
        let tokens = vec![101, 102, 103]; // Example token IDs
        for token in tokens {
            if let Some(text) = stream.next_token(token)? {
                println!("Generated text: {}", text);
            }
        }

        Ok(())
    }
}