use actix_web::{Responder, HttpResponse, web, HttpServer, App};
use std::fs;

pub async fn index() -> impl Responder {    
    HttpResponse::Ok().body("Congratulations! If you're reading this, Tigre tile server is running!")
}

pub async fn map(path: web::Path<(String, f32, f32, f32)>) -> impl Responder {
    let (layer, z, x, y) = path.into_inner();

    if !fs::exists(format!("/tmp/tigre/{}.svg", layer)).unwrap() {
        return HttpResponse::NotFound().body(format!("Layer not found: {}", layer));
    }

    let result = web::block(move || {
        let svg = fs::read(format!("/tmp/tigre/{}.svg", layer)).unwrap();
        
        let width = 512.0;
        let height = 512.0;
        //let viewbox_size = 1000.0; // Match your SVG viewBox
        
        let scale = (2f32.powf(z) * width);
        let translate_x = width * x;
        let translate_y = height * y;

        let mut pixmap = resvg::tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();
        let tree = resvg::usvg::Tree::from_data(&svg, &resvg::usvg::Options::default()).unwrap();

        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default()
                .pre_scale(1.0, -1.0)  // Flip Y axis
                .post_scale(scale, scale)
                .post_translate(-translate_x, -translate_y),
            &mut pixmap.as_mut(),
        );

        pixmap.encode_png().unwrap()
    })
    .await;

    match result {
        Ok(png_data) => HttpResponse::Ok()
            .content_type("image/png")
            .body(png_data),
        Err(_) => HttpResponse::InternalServerError()
            .body("Failed to process tile")
    }
}

pub async fn start_tile_server() -> std::io::Result<actix_web::dev::Server> { 
    let server = HttpServer::new(move || App::new()
            .route("/", web::get().to(index))
            .route("/map/{layer}/{z}/{x}/{y}.png", web::get().to(map))
        )
        .bind(("127.0.0.1", 8081))?
        .run();

    Ok(server)
}