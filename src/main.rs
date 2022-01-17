#[macro_use]
extern crate dotenv_codegen;

mod google_schema;

use dotenv::dotenv;
use gcp_auth;
use google_schema::{
    ExtractedText, Feature as GoogleFeature, Image as GoogleImage, Request as GoogleRequest,
    RequestItem, Response as GoogleResponse,
};
use image_base64;
use loading::Loading;
use reqwest;
use std::path::PathBuf;
use std::process;
use std::{env, fs};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let mut loading = Loading::new();

    loading.start();

    let path_to_images = get_path();
    let image_paths =
        get_all_directory_images(Some(path_to_images)).expect("Could find any images!");

    if image_paths.len() == 0 {
        panic!("No valid images found in directory!")
    } else {
        println!("Found {} images", image_paths.len());
    }

    let base64_of_images: Vec<String> = image_paths
        .clone()
        .into_iter()
        .map(|image_path| load_image_convert_to_base64(&image_path))
        .collect();

    let mut extracted_collection: Vec<ExtractedText> = vec![];

    for chunk in base64_of_images.chunks(16) {
        loading.text(format!("processing {} images", chunk.len()));

        let request_body = prepare_request_body(&chunk.to_vec());
        let request_body_json =
            serde_json::to_string(&request_body).expect("Unable to build request body!");

        let api_key = load_google_clound_api_key().await?;

        let client = reqwest::Client::new();
        let res = client
            .post(dotenv!("GOOGLE_CLOUD_VISION_ENDPOINT"))
            .body(request_body_json)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let response_text = res.text().await?;
        let response: GoogleResponse = match serde_json::from_str(&response_text) {
            Err(_err) => {
                fs::write("response.log", response_text).expect("Unable to write log file.");
                eprintln!("{:?}", "An error occurred while parsing response. Please read response.log file for more information!");
                process::exit(0)
            }
            Ok(response) => response,
        };

        let extracted_text = response_to_output(&response);

        extracted_collection = [extracted_collection, extracted_text].concat();

        loading.success("processed!");
    }

    process_output(&extracted_collection)?;

    loading.end();

    Ok(())
}

pub async fn load_google_clound_api_key() -> Result<String, gcp_auth::Error> {
    let authentication_manager = gcp_auth::init().await?;
    let token = authentication_manager
        .get_token(&["https://www.googleapis.com/auth/cloud-vision"])
        .await?;

    Ok(token.as_str().to_string())
}

pub fn load_image_convert_to_base64(path: &str) -> String {
    let base64 = image_base64::to_base64(path);
    let prefix_encoded: Vec<&str> = base64.split(",").collect();

    prefix_encoded[1].to_string()
}

pub fn get_all_directory_images(
    path: Option<String>,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = path.unwrap_or(String::from("./"));
    let srcdir = PathBuf::from(&path);

    println!(
        "Looking for images in directory: {:?}",
        srcdir.canonicalize().unwrap()
    );

    let directory = fs::read_dir(&path)?;

    let supported_file_formats = vec![
        "jpeg", "png", "gif", "bmp", "webp", "raw", "ico", "pdf", "tiff",
    ];

    Ok(directory
        .map(|path| path.unwrap().path().display().to_string())
        .filter(|file| {
            let file_name_extension = file.split(".").collect::<Vec<&str>>();
            let extension = file_name_extension.last().unwrap();

            supported_file_formats.contains(extension)
        })
        .collect::<Vec<String>>())
}

pub fn get_path() -> String {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return String::from("./");
    }

    println!("{}", args[1].to_owned());
    args[1].to_owned()
}

pub fn prepare_request_body(base64_images: &Vec<String>) -> GoogleRequest {
    GoogleRequest {
        requests: base64_images
            .into_iter()
            .map(|base64_image| RequestItem {
                image: GoogleImage {
                    content: base64_image.to_string(),
                },
                features: vec![GoogleFeature {
                    r#type: String::from("TEXT_DETECTION"),
                }],
            })
            .collect::<Vec<RequestItem>>(),
    }
}

pub fn response_to_output(response: &GoogleResponse) -> Vec<ExtractedText> {
    response
        .clone()
        .responses
        .iter()
        .map(|response_item| ExtractedText {
            text: response_item.fullTextAnnotation.text.clone(),
        })
        .collect::<Vec<ExtractedText>>()
}

pub fn process_output(
    extracted_text: &Vec<ExtractedText>,
) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string(&extracted_text).expect("Unable to build output!");

    let args: Vec<String> = env::args().collect();

    if args.len() == 3 {
        fs::write(args[2].to_string(), json).expect("Unable to write file");

        println!("Completed extraction. Output written to file: {}", args[2].to_string());
        process::exit(0)
    }

    println!("{:#?}", json);

    process::exit(0)
}
