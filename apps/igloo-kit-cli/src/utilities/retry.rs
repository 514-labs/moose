pub async fn retry<E, T, F>(
    action: impl Fn() -> F,
    should_retry: fn(u32, &E) -> bool,
    delay: tokio::time::Duration,
) -> Result<T, E>
where
    F: std::future::Future<Output = Result<T, E>>,
{
    let mut i = 0;
    loop {
        match action().await {
            Ok(res) => return Ok(res),
            Err(err) => {
                if should_retry(i, &err) {
                    i += 1;
                    tokio::time::sleep(delay).await;
                } else {
                    return Err(err);
                }
            }
        }
    }
}
