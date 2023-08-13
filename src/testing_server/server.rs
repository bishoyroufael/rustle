use std::path::Path;
use std::process::Command;
use actix_web::{HttpRequest, 
                HttpResponse, 
                Responder, 
                HttpServer,
                App,
                get,
                route
            };
use actix_files::NamedFile;


#[route("/download", method="GET", method="HEAD")]
async fn download(request : HttpRequest) -> impl Responder {
    let file_path = "file.temp.bin";

    // If file wasn't created i.e first time
    if !Path::new(file_path).is_file(){
        let status = Command::new("dd")
                                .args(["if=/dev/urandom", &format!("of={}", file_path), "bs=64M", "count=1"])
                                .status();        
        if status.is_err(){
            return HttpResponse::InternalServerError().body(format!("Couldn't create the temp file, err : {}", status.err().unwrap().to_string()))
        }
    }

    let response = NamedFile::open(file_path).unwrap().into_response(&request);

    return response;
}


#[get("/")]
async fn root() -> HttpResponse {
    HttpResponse::Ok().body("Ok!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("-> Server starting ..");

    HttpServer::new(|| {
        App::new()
            .service(root)
            .service(download)
    })
    .bind(("127.0.0.1", 5555))?
    .run()
    .await

}