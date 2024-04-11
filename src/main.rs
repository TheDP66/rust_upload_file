use actix_files::{Files, NamedFile};
use actix_multipart::Multipart;
use actix_web::{
    error, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use futures::{StreamExt, TryStreamExt};
use std::{fs::File, io::Write, path::PathBuf};

#[derive(Debug)]
struct FormData {
    name: String,
    file: Option<Vec<u8>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .service(Files::new("/images", "storage").show_files_listing())
            .route("/api/check", web::get().to(health_checker_handler))
            .route("/image/{title}", web::get().to(get_image))
            .route("/user", web::post().to(upload_file))
    })
    .bind(("127.0.0.1", 8090))?;

    server.run().await
}

async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "100% healthy";

    HttpResponse::Ok().json(serde_json::json!({"status": "success", "message": MESSAGE}))
}

async fn upload_file(mut payload: Multipart, req: HttpRequest) -> Result<HttpResponse, Error> {
    let mut form_data = FormData {
        name: String::new(),
        file: None,
    };

    let dir: &str = "./storage/";
    let user_id = uuid::Uuid::new_v4().to_string();

    while let Ok(Some(mut field)) = payload.try_next().await {
        println!("field: {:?}", field);

        let mut buffer = Vec::new();

        while let Some(chunk) = field.next().await {
            let data = chunk?;
            buffer.extend_from_slice(&data);
        }

        if field.name() == "name" {
            form_data.name = match String::from_utf8(buffer.clone()) {
                Ok(name) => name,
                Err(_) => "".to_owned(),
            };
        }

        if field.name() == "file" {
            form_data.file = Some(buffer.clone());

            let filename = field.content_disposition().get_filename().unwrap();

            if let Some(extension) = filename.rfind(".") {
                let extension = &filename[extension..];

                let saved_name = format!("{}{}", user_id, extension);

                let destination: String = format!("{}{}", dir, saved_name,);

                print!("dest: {}", destination);

                let mut file = File::create(destination)?;
                file.write_all(&form_data.file.unwrap())?;
            };
        }
    }

    println!("form_data.name: {:?}", form_data.name);

    Ok(HttpResponse::Ok().finish())
}

async fn get_image(title: web::Path<String>) -> Result<NamedFile, Error> {
    let file_path: PathBuf = format!("storage/{}", title).parse().unwrap();

    // HttpResponse::Ok()
    //     .content_type("image/jpeg")
    //     .body("storage/IMG-20240202-WA0000.jpg");

    match NamedFile::open(file_path) {
        Ok(file) => Ok(file),
        Err(e) => Err(error::ErrorInternalServerError(e)),
    }
}
