use ollama_rs::{
    generation::completion::request::GenerationRequest,
    Ollama,
};
use tokio_stream::{Stream, StreamExt};

pub struct OllamaAgent {
    model: String,
    host: String,
    port: u16,
    system_prompt: String,
    ollama: Ollama,
}

impl OllamaAgent {
    pub fn new(model: &str, host: &str, port: &u16) -> Self {
        let host = host.to_string();
        let port = *port;
        let ollama = Ollama::new(host.clone(), port);

        Self {
            model: model.to_string(),
            host,
            port,
            system_prompt: "你是一个使用中文作为主要语言的问答助手。".to_string(),
            ollama,
        }
    }

    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    pub fn with_host_port(mut self, host: &str, port: u16) -> Self {
        self.host = host.to_string();
        self.port = port;
        self.ollama = Ollama::new(host.to_string(), port);
        self
    }

    pub async fn generate_response(&self, user_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let full_prompt = format!("{}\n\n{}", self.system_prompt, user_prompt);
        let request = GenerationRequest::new(self.model.clone(), full_prompt);

        let mut stream = self.ollama.generate_stream(request).await?;
        let mut response_output = String::new();

        while let Some(res) = stream.next().await {
            let responses = res?;
            for resp in responses {
                response_output.push_str(&resp.response);
            }
        }

        Ok(response_output)
    }

    pub async fn generate_stream(&self, user_prompt: &str) -> Result<impl Stream<Item = String>, Box<dyn std::error::Error>> {
        let full_prompt = format!("{}\n\n{}", self.system_prompt, user_prompt);
        let request = GenerationRequest::new(self.model.clone(), full_prompt);

        let stream = self.ollama.generate_stream(request).await?;
        Ok(stream.map(|res| {
            match res {
                Ok(responses) => {
                    let mut combined = String::new();
                    for resp in responses {
                        combined.push_str(&resp.response);
                    }
                    combined
                },
                Err(_) => String::new(),
            }
        }))
    }
}