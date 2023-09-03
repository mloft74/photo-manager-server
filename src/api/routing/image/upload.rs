use std::io;

use axum::{
    extract::{
        multipart::{Field, MultipartError},
        DefaultBodyLimit, Multipart,
    },
    routing::post,
    BoxError, Router,
};
use futures::{Stream, TryStreamExt};
use hyper::{body::Bytes, StatusCode};
use serde::Serialize;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tower_http::limit::RequestBodyLimitLayer;

use crate::{
    api::{
        image_dimensions::{self, FetchImageDimensionsError},
        routing::ApiError,
        IMAGES_DIR,
    },
    domain::{
        actions::image::{FetchImage, SaveImage},
        models::Image,
        screensaver::Screensaver,
    },
};

pub fn make_upload_router(
    image_mngr: impl 'static + Clone + Send + Sync + FetchImage + SaveImage,
    ss_mngr: impl 'static + Clone + Send + Sync + Screensaver,
) -> Router {
    Router::new()
        .route(
            "/upload",
            post(|body| upload_image(body, image_mngr, ss_mngr)),
        )
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            250 * 1024 * 1024, /* 250mb */
        ))
}

#[derive(Serialize)]
enum UploadImageError {
    FileFieldErr(FileFieldValidationError),
    ImageAlreadyExists,
    FailedToFetchDimensions(FetchImageDimensionsError),
    GeneralError(String),
}

impl ApiError for UploadImageError {}

// Handler that accepts a multipart form upload and streams each field to a file.
async fn upload_image(
    mut multipart: Multipart,
    image_mngr: impl FetchImage + SaveImage,
    mut ss_mngr: impl Screensaver,
) -> Result<(), (StatusCode, String)> {
    let (file_name, file_field) = validate_field(multipart.next_field().await).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            UploadImageError::FileFieldErr(e).to_json_string(),
        )
    })?;

    let existing_image = image_mngr.fetch_image(&file_name).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadImageError::GeneralError(e).to_json_string(),
        )
    })?;
    if existing_image.is_some() {
        return Err((
            StatusCode::BAD_REQUEST,
            UploadImageError::ImageAlreadyExists.to_json_string(),
        ));
    }

    stream_to_file(&file_name, file_field)
        .await
        .map_err(|(s, e)| (s, e.to_json_string()))?;

    let (image_width, image_height) = image_dimensions::fetch_image_dimensions(&file_name)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                UploadImageError::FailedToFetchDimensions(e).to_json_string(),
            )
        })?;

    tracing::debug!("image dimensions: {} x {}", image_width, image_height);

    let image = Image {
        file_name,
        width: image_width,
        height: image_height,
    };

    image_mngr.save_image(&image).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadImageError::GeneralError(e).to_json_string(),
        )
    })?;

    ss_mngr.insert(image);

    Ok(())
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(
    file_name: &str,
    stream: S,
) -> Result<(), (StatusCode, UploadImageError)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new(IMAGES_DIR).join(file_name);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadImageError::GeneralError(err.to_string()),
        )
    })
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

#[derive(Serialize)]
enum FileFieldValidationError {
    FieldErr(String),
    MissingField,
    MissingFieldName,
    MissingFileName,
    InvalidFileName,
    WrongFieldName { expected: String, actual: String },
}

fn validate_field(
    field_res: Result<Option<Field>, MultipartError>,
) -> Result<(String, Field), FileFieldValidationError> {
    let field = field_res.map_err(|e| FileFieldValidationError::FieldErr(e.to_string()))?;
    let field = field.ok_or(FileFieldValidationError::MissingField)?;
    let field_name = field
        .name()
        .ok_or(FileFieldValidationError::MissingFieldName)?;
    let file_name = field
        .file_name()
        .ok_or(FileFieldValidationError::MissingFileName)?;

    if !path_is_valid(file_name) {
        return Err(FileFieldValidationError::InvalidFileName);
    }

    let expected_name = "";
    if field_name != expected_name {
        return Err(FileFieldValidationError::WrongFieldName {
            expected: expected_name.to_string(),
            actual: field_name.to_string(),
        });
    }

    Ok((file_name.to_string(), field))
}
