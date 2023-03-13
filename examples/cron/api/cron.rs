use slack_morphism::{errors::SlackClientError, prelude::*};
use vercel_runtime::{run, Body, Error, Request, Response, StatusCode};

#[derive(Debug, Clone)]
pub struct SlackMessage {}

impl SlackMessageTemplate for SlackMessage {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new().with_blocks(slack_blocks![some_into(
            SlackSectionBlock::new().with_text(md!("你好， 世界！".to_owned()))
        )])
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

    pub async fn handler(&self, _req: Request) -> Result<Response<Body>, Error> {
        let message = SlackMessage {};

        self.post_message(&message, "#general").await?;

        Ok(Response::builder().status(StatusCode::OK).body(().into())?)
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let slack_client = SlackClient::new(SlackClientHyperConnector::new());
    let token_value: SlackApiTokenValue = std::env::var("SLACK_API_TOKEN")?.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let slack = slack_client.open_session(&token);

    let lambda = Lambda { slack };

    run(|e| lambda.handler(e)).await
}
