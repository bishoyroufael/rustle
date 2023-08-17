use futures::future::join_all;
use bytes::{Bytes, BytesMut};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::task::JoinHandle;
use tokio::task;
use reqwest::{header::{HeaderValue, RANGE, CONTENT_DISPOSITION, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE}, StatusCode};
use url::Url;
use std::{str::FromStr, time::Duration};
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::sync::Arc;
use super::io::write_bytes_to_file_in_dir;
use std::time::Instant;

/// Represents the level of support for partial requests.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SupportPartialRequest {
    /// Indicates that partial requests are supported.
    Yes,
    /// Indicates that partial requests are not supported.
    No,
    /// Indicates an unknown level of support for partial requests.
    #[default]
    Unknown
}

/// A wrapper struct for a valid URL.
/// It provides a convenient way to create and access a URL.
#[derive(Debug, Clone)]
pub struct ValidUrl(Url);

impl ValidUrl {
    /// Creates a new ValidUrl instance from a string representation of a URL.
    /// Returns Ok with the ValidUrl if the URL is valid, or a ParseError if it's not.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice representing a URL.
    pub fn new(url: &str) -> Result<Self, url::ParseError> {
        let parsed_url = Url::from_str(url)?;
        Ok(ValidUrl(parsed_url))
    }

    /// Returns the string representation of the URL.
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}


/// ResponseHeaderInfo represents the header information received in response to a request.
#[derive(Debug, Default, Clone)]
pub struct ResponseHeaderInfo {
    pub support_partial: SupportPartialRequest,   // Indicates whether partial downloading is supported
    pub content_length: Option<u64>,              // Length of the content in bytes
    pub content_type: Option<String>,             // MIME type of the content
    pub file_name: Option<String>,                // Name of the file
}

/// PartDownloadInfo represents information about a downloaded part of a file.
#[derive(Debug, Clone, Copy)]
pub struct PartDownloadInfo {
    pub downloaded_bytes: usize,  // Number of bytes downloaded for this part
    pub download_speed: f64,      // Download speed in bytes per second for this part
}

/// RustleDownloaderInner represents the internal state of the RustleDownloader.
#[derive(Debug, Default)]
struct RustleDownloaderInner {
    pub url: Option<ValidUrl>,                    // URL for downloading
    pub out_dir: Option<PathBuf>,                 // Output directory for downloaded files
    pub max_parallel_connections: u8,             // Number of parallel connections allowed for partial downloading
    pub get_headers_info: Option<ResponseHeaderInfo>,  // Header information received in response to a request
    pub progress_bar: Option<indicatif::ProgressBar>,   // Progress bar for tracking download progress
    pub progress_vec: Vec<PartDownloadInfo>,      // Vector containing information about downloaded parts
    pub download_status: DownloadStatus,          // Current download status
}

/// DownloadStatus represents the status of a download.
#[derive(Default, Debug, Clone, Copy)]
pub enum DownloadStatus {
    #[default]
    Idle,       // Download is idle
    Downloading,    // Download is in progress
    Paused,     // Download is paused
    Done,       // Download is completed
    Error,      // Download encountered an error
}

/// RustleDownloader represents a downloader tool for downloading files.
#[derive(Debug, Clone, Default)]
pub struct RustleDownloader {
    inner: Arc<Mutex<RustleDownloaderInner>>,
}


impl RustleDownloader {
    /// Extracts information from the response headers and returns a `Result` containing the extracted information
    ///
    /// # Arguments
    ///
    /// * `self` - A reference to the `RustleDownloader` struct
    /// * `response` - A reference to the `reqwest::Response` object
    ///
    /// # Returns
    ///
    /// * A `Result` containing the extracted `ResponseHeaderInfo` or an error message
    async fn extract_header_info(self: &RustleDownloader, response: &reqwest::Response) -> Result<ResponseHeaderInfo, String> {

        let response_headers = response.headers();
        let mut res_headers_info= ResponseHeaderInfo::default();

        // Content-Length 
        if let Some(cl_value) = response_headers.get(CONTENT_LENGTH) {
            let cl_string = cl_value.to_str().map_err(|e| format!("An error occurred while parsing the content-length: {}", e))?;
            let content_bytes = cl_string.parse().map_err(|e| format!("Content-Length isn't a valid number, error : {}", e))?;
            res_headers_info.content_length = Some(content_bytes);
        }

        // Accept-Ranges
        if let Some(ar_value) = response_headers.get(ACCEPT_RANGES) {
            let ar_string = ar_value.to_str().map_err(|e| format!("An error occurred while parsing the header value: {}", e))?;
            if ar_string.contains("bytes") {
                res_headers_info.support_partial = SupportPartialRequest::Yes;
            } else {
                res_headers_info.support_partial = SupportPartialRequest::No;
            }
        }

        // Content-Type
        if let Some (ct_value) = response_headers.get(CONTENT_TYPE){
            let content_type = ct_value
            .to_str()
            .map_err(|err| format!("Cannot convert content-disposition header value to string, err: {}", err))?;
            res_headers_info.content_type = Some(content_type.to_string());
        }

        // Content-Disposition
        // 1. Using the content-disposition field
        if let Some (cd_value) = response_headers.get(CONTENT_DISPOSITION){
            let filename = cd_value
                .to_str()
                .map_err(|err| format!("Cannot convert content-disposition header value to string, err: {}", err))?;

            let filename = filename
                .split(';')
                .find(|part| part.trim().starts_with("filename="))
                .and_then(|filename_part| filename_part.trim().split('=').nth(1))
                .map(|filename| filename.trim_matches('"').trim_matches('\''))
                .ok_or("Filename not found in content-disposition header.")?;

            res_headers_info.file_name = Some(filename.to_string());
        }
        // 2. Using the file path itself 
        else if let Some(filename) = response.url().path_segments().and_then(|segments| segments.last()) {
            res_headers_info.file_name = Some(filename.to_string());
        }
        else {
            // Default name in case the name cannot be detected
            res_headers_info.file_name = Some(String::from("download_file"));
        }

        return Ok(res_headers_info);

    }

    /// Initializes the RustleDownloader by performing an initial GET request.
    /// The response headers should provide information about the support for 
    /// partial requests and the download file information.
    ///
    /// # Returns
    /// Returns a `Result` indicating whether the initialization was successful (`Ok(true)`)
    /// or an error message (`Err(String)`).
    pub async fn init(self: &mut RustleDownloader) -> Result<bool, String> {
        /*
            Do an initial GET request

            -> response headers should give a hint about the support of 
            partial requests and the information of the download file.
        */

        let mut inner = self.inner.lock().await;

        assert!(inner.url.is_some(), "No valid url was supplied");
        assert!(inner.out_dir.is_some(), "No valid out_dir was supplied");

        let client = reqwest::Client::new();
        let response_get = client.get(inner.url.as_ref().unwrap().as_str()).timeout(Duration::from_secs(3)).send().await.map_err(|op| op.to_string())?;

        
        let get_info  = self.extract_header_info(&response_get).await?;
        inner.get_headers_info = Some(get_info);

        return Ok(true);
    }

    /// Pauses the RustleDownloader, changing the download status to `Paused`.
    pub async fn pause(self: &RustleDownloader) -> () {
        self.inner.lock().await.download_status = DownloadStatus::Paused;
    }

    /// Resumes the RustleDownloader, changing the download status to `Downloading`.
    pub async fn resume(self: &RustleDownloader) -> () {
        self.inner.lock().await.download_status = DownloadStatus::Downloading;
    }

    /// Retrieves the current download status of the RustleDownloader.
    pub async fn get_status(self: &RustleDownloader) -> DownloadStatus {
        self.inner.lock().await.download_status
    }

    /// Retrieves the file information obtained from the response headers.
    /// Returns `Some(ResponseHeaderInfo)` if the information is available, otherwise `None`.
    pub async fn get_file_info(self: &RustleDownloader) -> Option<ResponseHeaderInfo>{
        self.inner.lock().await.get_headers_info.clone()
    }

    /// Retrieves a vector of `PartDownloadInfo` representing the progress of each download part.
    /// This vector contains information such as the start and end range of each part and the number
    /// of bytes downloaded for each part.
    pub async fn get_progress_vec(self: &RustleDownloader) -> Vec<PartDownloadInfo> {
        self.inner.lock().await.progress_vec.clone()
    }


    /* Setters */
    /// Sets the URL for the RustleDownloader.
    ///
    /// # Arguments
    ///
    /// * `url` - A string slice containing the URL to be set.
    ///
    /// Returns an error if the provided URL is invalid.
    pub async fn set_url(self: &mut RustleDownloader, url: &str) -> Result<&RustleDownloader, String> {
        let url = ValidUrl::new(&url).map_err(|e| e.to_string())?;
        self.inner.lock().await.url = Some(url);
        return Ok(self);
    }

    /// Sets the output directory for the RustleDownloader.
    ///
    /// # Arguments
    ///
    /// * `out_dir` - A string slice containing the path of the output directory.
    ///
    /// Returns an error if the provided directory path is invalid.
    pub async fn set_out_dir(self: &mut RustleDownloader, out_dir: &str) -> Result<&RustleDownloader, String> {
        let out_dir = PathBuf::from_str(&out_dir).map_err(|e| e.to_string())?;
        self.inner.lock().await.out_dir = Some(out_dir);
        return Ok(self);
    }

    /// Creates a new instance of RustleDownloader.
    ///
    /// # Arguments
    ///
    /// * `max_parallel_connections` - The maximum number of parallel connections for downloading.
    ///
    /// Returns an error if the maximum number of parallel connections is zero.
    pub fn new (max_parallel_connections : u8) -> Result<RustleDownloader, String>{
        return Ok(
            RustleDownloader 
                { 
                    inner: Arc::new(Mutex::new(RustleDownloaderInner 
                        {
                         url : None,
                         out_dir : None,
                         max_parallel_connections,
                         get_headers_info: None, 
                         progress_bar: None,
                         progress_vec: Vec::new(),
                         download_status: DownloadStatus::Idle
                        })),
                })
    }

    /// Downloads a file asynchronously from a given URL using multiple parallel connections.
    /// If `with_progress_bar` is `true`, a progress bar will be displayed during the download process.
    ///
    /// # Arguments
    ///
    /// * `self` - The RustleDownloader object reference.
    /// * `with_progress_bar` - A boolean value indicating whether to display a progress bar.
    ///
    /// # Returns
    ///
    /// * `Result<bool, String>` - A Result indicating whether the download was successful or an error occurred.
    pub async fn download(self: &RustleDownloader, with_progress_bar: bool) -> Result<bool, String> {
        {
            let inner = self.inner.lock().await;

            assert!(inner.url.is_some(), "No valid url was supplied");
            assert!(inner.out_dir.is_some(), "No valid out_dir was supplied");
        }

        // Get required variables from inner
        let get_headers_info = {
            let inner = self.inner.lock().await;
            inner.get_headers_info.clone()
        };
        let mut num_parts = {
            let inner = self.inner.lock().await;
            inner.max_parallel_connections.clone() as u64
        };

        match get_headers_info.as_ref() {
            Some(headers_info) => {
                
                // if partial downloads isn't allowed OR content-length isn't defined
                if headers_info.support_partial != SupportPartialRequest::Yes || headers_info.content_length.is_none() {
                    num_parts = 1;
                }
                

                let content_length = headers_info.content_length.unwrap_or(0);
                let inc = content_length / num_parts;
                
                // Init the progress vector
                // Init the progress bar
                {
                    let mut inner = self.inner.lock().await;
                    inner.progress_vec = vec![PartDownloadInfo { downloaded_bytes: 0, download_speed: 0.0 }; num_parts as usize];
                    
                    if with_progress_bar {
                        let pb = ProgressBar::new(content_length);
                        pb.set_style(
                            ProgressStyle::default_bar()
                                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} | {msg} ({eta})")
                                .progress_chars("#>-"),
                        );
                        inner.progress_bar = Some(pb);
                    }
                } 

                // Update downloading status
                self.inner.lock().await.download_status = DownloadStatus::Downloading;

                let mut tasks : Vec<JoinHandle<Result<Bytes, String>>> = Vec::new();
                for part in 0..num_parts {
                    let mut start_byte = part * inc;
                    let mut end_byte = (part + 1) * inc;
            
                    if part == num_parts - 1 && num_parts % 2 == 0 {
                        end_byte += 1;
                    }
                    if part != 0 {
                        start_byte += 1;
                    }
                    
                    let self_cloned = self.clone();
                    tasks.push(
                        task::spawn(async move {
                            self_cloned.download_part_from_url(start_byte, end_byte, part as usize).await
                        })
                    )
                };

                let download_results = join_all(tasks).await;
                let mut full_content = BytesMut::new();

                for result in download_results {
                    let future_result = result.unwrap_or(Err("Cannot unwrap future result task, something is wrong".to_string()));
                    let download_partial_buffer = future_result.unwrap_or_else(|_| Bytes::new());
                    full_content.extend_from_slice(&download_partial_buffer);
                }

                let full_content = bytes::Bytes::from(full_content);

                let file_name = headers_info.file_name.as_ref().unwrap();

                write_bytes_to_file_in_dir(&full_content, &file_name, &self.inner.lock().await.out_dir.as_ref().unwrap()).map_err(|op| op.to_string())?;

                // Finish and clear progress_bar if present
                if let Some(progress_bar) = self.inner.lock().await.progress_bar.as_ref() {
                    progress_bar.finish_and_clear();
                }

                self.inner.lock().await.download_status = DownloadStatus::Done;
             
                Ok(true)

            },
            None => {Err(String::from("Couldn't download the file, header info is missing"))},
        }

    }

    /// Downloads a specific part of a file from a given URL asynchronously.
    /// It uses the `start_byte` and `end_byte` parameters to specify the range of bytes to download.
    /// The `part_num` parameter is used for tracking progress and updating the progress bar.
    ///
    /// # Arguments
    ///
    /// * `self` - The RustleDownloader object reference.
    /// * `start_byte` - The starting byte index for the download range.
    /// * `end_byte` - The ending byte index for the download range.
    /// * `part_num` - The index of the part being downloaded.
    ///
    /// # Returns
    ///
    /// * `Result<Bytes, String>` - A Result containing the downloaded bytes or an error message.
    async fn download_part_from_url(self: &RustleDownloader, start_byte: u64, end_byte: u64, part_num: usize) -> Result<Bytes, String> {
        let client = reqwest::Client::new();
        let url = {
            let inner = self.inner.lock().await;
            inner.url.clone()
        };
        
        let range_header_value = HeaderValue::from_str(&format!("bytes={}-{}", start_byte, end_byte))
        .map_err(|e| format!("An error occured while creating the ranges header {}", e))?;
    
        let mut response = client
                    .get(url.unwrap().as_str())
                    .header(RANGE, range_header_value)
                    .send()
                    .await.map_err(|e| format!("An error occured while sending the download request, error : {}", e))?;

        if response.status() != StatusCode::PARTIAL_CONTENT {
            return Err(format!("Didn't recieve partial content, got status code : {} | content of response {}", response.status().as_str(), response.text().await.unwrap()));
        }

        let mut buffer = BytesMut::new();
        let start_time = Instant::now();

        let mut pause_duration = Duration::new(0,0);

        while let Some(chunk) = response.chunk()
                                            .await
                                            .unwrap_or(None) {


            
            // Wait if download was paused ..
            match self.get_status().await {
                DownloadStatus::Paused => {
                    let pause_time = Instant::now();
                    loop {
                        match self.get_status().await {
                            DownloadStatus::Downloading => {
                                // println!("Download is resumed, breaking");

                                pause_duration += pause_time.elapsed();
                                break;
                            },
                            _ => {}
                        }
                        // println!("Download was paused, looping until resumed");
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                },
                _ => {}
            };

            // println!("Extending buffer with chunk");

            buffer.extend_from_slice(&chunk);

            let elapsed_time = start_time.elapsed();
            
            let mut inner = self.inner.lock().await;
            // Add the number of downloaded chunks to track progress
            inner.progress_vec[part_num].downloaded_bytes += chunk.len();

            // Calculate the downloading speed (total pause time is subtracted if present)
            let downloading_speed = inner.progress_vec[part_num].downloaded_bytes as f64 / (elapsed_time.as_secs_f64() - pause_duration.as_secs_f64()); 
            inner.progress_vec[part_num].download_speed = downloading_speed;


            // Update progress bar if present
            if let Some(progress_bar) = inner.progress_bar.as_ref() {
                let downloading_speed : f64 = inner.progress_vec.iter().map(|item| item.download_speed).sum();
                progress_bar.inc(chunk.len() as u64);
                progress_bar.set_message(&format!(
                    "{:.2} MB/s",
                    downloading_speed / 1_000_000.0
                ));
            }
        } 

        let buffer = bytes::Bytes::from(buffer);

        Ok(buffer)
    }
}