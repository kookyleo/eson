use anyhow::Result;

async fn curl(url: &str) -> Result<String> {
    let mut resp = reqwest::get(url).await?;
    assert!(resp.status().is_success());
    Ok(resp.text().await?)
}
