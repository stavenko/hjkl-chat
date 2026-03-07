mod config;

#[cfg(test)]
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let server = actix_web::HttpServer::new(|| {
        actix_web::App::new()
    })
    .bind("127.0.0.1:8080")?
    .run();

    println!("Backend server starting on http://127.0.0.1:8080");
    server.await
}