use arti_pipes::executor::{ExecutionResult, Output, OutputMetadata, PromptExecutor, Token, TokenStream};
use futures::stream;

#[derive(Clone)]
pub struct EchoExecutor;

impl PromptExecutor for EchoExecutor {
    async fn execute_raw(
        &self,
        prompt: String,
    ) -> arti_pipes::error::Result<ExecutionResult<String>> {
        let echo_text = prompt.clone();

        let content_stream: TokenStream = Box::pin(stream::iter(
            echo_text
                .chars()
                .collect::<Vec<_>>()
                .chunks(4)
                .enumerate()
                .map(|(i, chunk)| {
                    let text: String = chunk.iter().collect();
                    Ok(Token {
                        content: text,
                        index: i,
                    })
                })
                .collect::<Vec<_>>(),
        ));

        let thinking_stream: TokenStream = Box::pin(stream::empty());

        let output_text = echo_text.clone();
        let output = tokio::spawn(async move {
            Ok(Output::new(
                output_text,
                OutputMetadata {
                    total_tokens: echo_text.len(),
                    generation_time_ms: 0,
                    model_id: "echo".to_string(),
                },
            ))
        });

        Ok(ExecutionResult {
            thinking_stream,
            content_stream,
            output,
        })
    }

    async fn execute<T: schemars::JsonSchema>(
        &self,
        prompt: String,
    ) -> arti_pipes::error::Result<ExecutionResult<String>> {
        self.execute_raw(prompt).await
    }
}
