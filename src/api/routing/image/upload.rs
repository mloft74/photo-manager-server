use std::io;

use axum::{
    extract::{
        multipart::{Field, MultipartError},
        DefaultBodyLimit, Multipart, State,
    },
    routing::post,
    BoxError, Router,
};
use futures::{Stream, TryStreamExt};
use hyper::{body::Bytes, StatusCode};
use image::io::Reader as ImageReader;
use serde::Serialize;
use serde_json::json;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;
use tower_http::limit::RequestBodyLimitLayer;

use crate::{
    api::IMAGES_DIR,
    domain::{
        actions::images::{ImageGetter, ImageSaver},
        models::Image,
    },
};

pub fn make_upload_router<TGetter: ImageGetter + 'static, TSaver: ImageSaver + 'static>(
    image_getter: TGetter,
    image_saver: TSaver,
) -> Router {
    Router::new()
        .route("/upload", post(upload_image))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(
            250 * 1024 * 1024, /* 250mb */
        ))
        .with_state(UploadState {
            getter: image_getter,
            saver: image_saver,
        })
}

#[derive(Clone)]
struct UploadState<TGetter: ImageGetter, TSaver: ImageSaver> {
    getter: TGetter,
    saver: TSaver,
}

#[derive(Serialize)]
struct UploadImageErrorWrapper<'a> {
    error: &'a UploadImageError,
}

#[derive(Serialize)]
enum UploadImageError {
    FileFieldErr(FileFieldValidationError),
    ImageAlreadyExists,
    FailedToFetchDimensions(GetImageDimensionsError),
    GeneralError(String),
}

impl UploadImageError {
    fn to_json_string(&self) -> String {
        serde_json::to_string(&UploadImageErrorWrapper { error: self }).unwrap_or_else(|e| {
            json!({
                "error": "jsonConverionFailed",
                "message": e.to_string(),
            })
            .to_string()
        })
    }
}

// Handler that accepts a multipart form upload and streams each field to a file.
async fn upload_image<TGetter: ImageGetter, TSaver: ImageSaver>(
    state: State<UploadState<TGetter, TSaver>>,
    multipart: Multipart,
) -> Result<(), (StatusCode, String)> {
    upload_image_inner(state, multipart).await
}

async fn upload_image_inner<TGetter: ImageGetter, TSaver: ImageSaver>(
    state: State<UploadState<TGetter, TSaver>>,
    mut multipart: Multipart,
) -> Result<(), (StatusCode, String)> {
    let (file_name, file_field) = validate_field(multipart.next_field().await).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            UploadImageError::FileFieldErr(e).to_json_string(),
        )
    })?;

    let existing_image = state.getter.get_image(&file_name).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadImageError::GeneralError(e.to_string()).to_json_string(),
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

    let (image_width, image_height) = get_image_dimensions(&file_name).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            UploadImageError::FailedToFetchDimensions(e).to_json_string(),
        )
    })?;

    tracing::debug!("image dimensions: {} x {}", image_width, image_height);

    state
        .saver
        .save_image(&Image {
            file_name,
            width: image_width,
            height: image_height,
        })
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                UploadImageError::GeneralError(e.to_string()).to_json_string(),
            )
        })?;

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

#[derive(Serialize)]
enum GetImageDimensionsError {
    ErrorOpeningImage(String),
    FailedToGetDimensions(String),
}

fn get_image_dimensions(file_name: &str) -> Result<(u32, u32), GetImageDimensionsError> {
    let path = std::path::Path::new(IMAGES_DIR).join(file_name);
    let image = ImageReader::open(path)
        .map_err(|e| GetImageDimensionsError::ErrorOpeningImage(e.to_string()))?;
    let dim = image
        .into_dimensions()
        .map_err(|e| GetImageDimensionsError::FailedToGetDimensions(e.to_string()))?;

    Ok(dim)
}
