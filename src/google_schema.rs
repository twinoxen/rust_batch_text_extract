use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
  pub responses: Vec<ResponseItem>,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct ResponseItem {
  pub textAnnotations: Vec<TextAnnotation>,
  pub fullTextAnnotation: FullTextAnnotation,
}

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct TextAnnotation {
  pub locale: Option<String>,
  pub description: String,
  pub boundingPoly: BoundingPoly
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundingPoly {
  pub vertices: Vec<Vertices>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vertices {
  pub x: i32,
  pub y: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FullTextAnnotation {
    pub text: String,
    pub pages: Vec<Page>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Page {
  pub width: i32,
  pub height: i32,
}

// Request
#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
  pub requests: Vec<RequestItem>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestItem {
  pub image: Image,
  pub features: Vec<Feature>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Feature {
  pub r#type: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Image {
  pub content: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtractedText {
  pub text: String
}