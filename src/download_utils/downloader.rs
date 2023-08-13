use futures::future::join_all;
use bytes::{Bytes, BytesMut};
use indicatif::{ProgressBar, ProgressStyle};
use tokio::task::JoinHandle;
use tokio::task;
use reqwest::{header::{HeaderValue, RANGE, CONTENT_DISPOSITION, ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_TYPE}, StatusCode};
use url::Url;
use std::{str::FromStr, default, time::Duration};
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::sync::Arc;
use super::io::write_bytes_to_file_in_dir;
use std::time::Instant;


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SupportPartialRequest {
    Yes,
    No,
    #[default]
    Unknown
}

#[derive(Debug, Clone)]
pub struct ValidUrl(Url);

impl ValidUrl {
    pub fn new(url: &str) -> Result<Self, url::ParseError> {
        let parsed_url = Url::from_str(url)?;
        Ok(ValidUrl(parsed_url))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Default, Clone)]
pub struct ResponseHeaderInfo {
    pub support_partial : SupportPartialRequest,
    pub content_length : Option<u64>,
    pub content_type : Option<String>,
    pub file_name : Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct PartDownloadInfo {
    pub downloaded_bytes : usize,
    pub download_speed : f64
}

#[derive(Debug, Default)]
// https://stackoverflow.com/questions/76137948/using-mut-self-in-an-async-move-block
struct  RustleDownloaderInner {
    // Url for downloading
    pub url : Option<ValidUrl>,
    // Output directory
    pub out_dir : Option<PathBuf>,
    // Num parallel connections if partial downloading is allowed
    pub max_parallel_connections : u8,
    // Header information to infer details about the file 
    get_headers_info : Option<ResponseHeaderInfo>,
    // Progress bar
    progress_bar : Option<indicatif::ProgressBar>,
    // Vector containing (bytes, speed) per part
    progress_vec : Vec<PartDownloadInfo>,

    // Downloading status
    download_status: DownloadStatus
}

#[derive(Default, Debug, Clone, Copy)]
pub enum DownloadStatus {
    #[default]
    Idle,
    Downloading,
    Paused,
    Done,
    Error
}

#[derive(Debug, Clone, Default)]
pub struct RustleDownloader {
    inner: Arc<Mutex<RustleDownloaderInner>>,
}


impl RustleDownloader {

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

    pub async fn pause(self: &RustleDownloader) -> () {
        self.inner.lock().await.download_status = DownloadStatus::Paused;
    }

    pub async fn resume(self: &RustleDownloader) -> () {
        self.inner.lock().await.download_status = DownloadStatus::Downloading;
    }

    pub async fn get_status(self: &RustleDownloader) -> DownloadStatus {
        self.inner.lock().await.download_status
    }

    /* Getters */
    pub async fn get_file_info(self: &RustleDownloader) -> Option<ResponseHeaderInfo>{
        self.inner.lock().await.get_headers_info.clone()
    }

    pub async fn get_progress_vec(self: &RustleDownloader) -> Vec<PartDownloadInfo> {
        self.inner.lock().await.progress_vec.clone()
    }


    /* Setters */
    pub async fn set_url(self: &mut RustleDownloader, url: &str) -> Result<&RustleDownloader, String> {
        let url = ValidUrl::new(&url).map_err(|e| e.to_string())?;
        self.inner.lock().await.url = Some(url);
        return Ok(self);
    }

    pub async fn set_out_dir(self: &mut RustleDownloader, out_dir: &str) -> Result<&RustleDownloader, String> {
        let out_dir = PathBuf::from_str(&out_dir).map_err(|e| e.to_string())?;
        self.inner.lock().await.out_dir = Some(out_dir);
        return Ok(self);
    }

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
        let mut start_time = Instant::now();

        while let Some(chunk) = response.chunk()
                                            .await
                                            .unwrap_or(None) {


            // Wait if download was paused ..
            match self.get_status().await {
                DownloadStatus::Paused => {
                    loop {
                        match self.get_status().await {
                            DownloadStatus::Downloading => {
                                // println!("Download is resumed, breaking");

                                // re-initialize the start_time for resuming the download
                                // start_time = Instant::now();
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
            // Calculate the downloading speed and save to progress vec
            let downloading_speed = inner.progress_vec[part_num].downloaded_bytes as f64 / elapsed_time.as_secs_f64(); 
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