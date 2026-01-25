mod families;
mod jira;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let workspace = std::env::var("JIRA_WORKSPACE").unwrap_or_else(|e| {
        tracing::error!("JIRA_WORKSPACE environment variable not set: {}", e);
        panic!("JIRA_WORKSPACE environment variable not set");
    });
    let username = std::env::var("JIRA_USERNAME").unwrap_or_else(|e| {
        tracing::error!("JIRA_USERNAME environment variable not set: {}", e);
        panic!("JIRA_USERNAME environment variable not set");
    });
    let token = std::env::var("JIRA_TOKEN").unwrap_or_else(|e| {
        tracing::error!("JIRA_TOKEN environment variable not set: {}", e);
        panic!("JIRA_TOKEN environment variable not set");
    });

    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let jira = jira::Jira::new(&workspace, &username, &token)
        .serve(transport)
        .await?;
    jira.waiting().await?;
    Ok(())
}
