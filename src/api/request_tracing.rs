use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::{StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;

pub async fn print_request_response(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let uri = parts.uri.clone();
    let bytes = buffer_and_print_request(&uri, body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print_response(&parts.status, &uri, body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print_request<B>(uri: &Uri, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} request body: {}", uri, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} request body = {:?}", uri, body);
    }

    Ok(bytes)
}

async fn buffer_and_print_response<B>(
    status_code: &StatusCode,
    uri: &Uri,
    body: B,
) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {} response body: {}", uri, err),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{} {} response body = {:?}", uri, status_code, body);
    }

    Ok(bytes)
}
