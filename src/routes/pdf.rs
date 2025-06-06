use axum::{
    extract::Multipart,
    response::{IntoResponse, Response},
    http::{header, StatusCode},
};
use std::io::Write;
use tempfile::NamedTempFile;
use pdf_extract::extract_text;
use tracing::{info, error};

fn clean_blank_lines(text: &str) -> String {
    text.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<&str>>().join("\n")
}

pub async fn upload_pdf_handler(mut multipart: Multipart) -> Response {
    // Process the multipart form data
    if let Ok(Some(field)) = multipart.next_field().await {
        // Verify it's a PDF file
        let content_type = field.content_type().unwrap_or("").to_string();
        if !content_type.eq("application/pdf") {
            return (
                StatusCode::BAD_REQUEST,
                "Invalid file type. Only PDF files are accepted."
            ).into_response();
        }

        // Read the file data
        match field.bytes().await {
            Ok(data) => {
                // Create a temporary file
                let mut temp_file = match NamedTempFile::new() {
                    Ok(file) => file,
                    Err(e) => {
                        error!("Failed to create temporary file: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to process file"
                        ).into_response();
                    }
                };

                // Write the PDF data to the temporary file
                if let Err(e) = temp_file.write_all(&data) {
                    error!("Failed to write to temporary file: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to process file"
                    ).into_response();
                }

                // Extract text from the PDF
                match extract_text(temp_file.path()) {
                    Ok(text) => {
                        info!("Successfully extracted text from PDF");
                        
                        // Create response headers for text file download
                        let headers = [
                            (header::CONTENT_TYPE, "text/plain; charset=utf-8"),
                            (
                                header::CONTENT_DISPOSITION,
                                "attachment; filename=\"extracted_text.txt\""
                            ),
                        ];

                        return (StatusCode::OK, headers, clean_blank_lines(&text)).into_response();
                    }
                    Err(e) => {
                        error!("Failed to extract text from PDF: {}", e);
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to extract text from PDF"
                        ).into_response();
                    }
                }
            }
            Err(e) => {
                error!("Failed to read file data: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to read file data"
                ).into_response();
            }
        }
    }

    (StatusCode::BAD_REQUEST, "No file provided").into_response()
}
