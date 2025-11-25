use back_end::api_docs::ApiDoc;
use utoipa::OpenApi;

fn main() {
    println!(
        "{}",
        ApiDoc::openapi()
            .to_json()
            .expect("Failed to generate JSON")
    );
}
