use slack_morphism::{errors::SlackClientError, prelude::*};
use vercel_runtime::{
    lambda_http::{
        http::{Error, StatusCode},
        Error as LambdaError, Response,
    },
    run, IntoResponse, Request,
};

#[derive(Debug, Clone)]
pub struct SlackMessage {}

impl SlackMessageTemplate for SlackMessage {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new().with_blocks(slack_blocks![some_into(SlackContextBlock::new(
            slack_blocks![some(md!("你好， 世界！"))]
        ))])
    }
}

struct Lambda<'a, T: SlackClientHttpConnector + Send + Sync> {
    slack: SlackClientSession<'a, T>,
}

impl<T: SlackClientHttpConnector + Send + Sync> Lambda<'_, T> {
    pub async fn post_message(
        &self,
        message: &impl SlackMessageTemplate,
        channel: &str,
    ) -> Result<SlackApiChatPostMessageResponse, SlackClientError> {
        let post_chat_req =
            SlackApiChatPostMessageRequest::new(channel.into(), message.render_template());

        self.slack.chat_post_message(&post_chat_req).await
    }

    pub async fn handler(&self, _req: Request) -> Result<impl IntoResponse, Error> {
        let message = SlackMessage {};

        self.post_message(&message, "#general").await.unwrap();

        let response = Response::builder().status(StatusCode::OK).body(())?;
        Ok(response)
    }
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let slack_client = SlackClient::new(SlackClientHyperConnector::new());
    let token_value: SlackApiTokenValue = std::env::var("SLACK_API_TOKEN")?.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let slack = slack_client.open_session(&token);

    let lambda = Lambda { slack };

    run(|e| lambda.handler(e)).await?;
    Ok(())
}
