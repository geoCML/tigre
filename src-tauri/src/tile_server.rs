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
        let svg = fs::read_to_string(format!("/tmp/tigre/{}.svg", layer)).unwrap();
        
        let width = 128.0;
        let height = 128.0;
        let translate_x = -width * x;
        let translate_y = -height * y;
        
        let scale = 2f32.powf(z);
        
        let mut pixmap = resvg::tiny_skia::Pixmap::new(width as u32, height as u32).unwrap();
        let tree = resvg::usvg::Tree::from_data(svg.as_bytes(), &resvg::usvg::Options::default()).unwrap();

        // Transform: scale to zoom level and translate to position
        resvg::render(
            &tree,
            resvg::tiny_skia::Transform::default()
                .pre_scale(1.0, -1.0) 
                .pre_translate(-width / 2., height / 2.)
                .post_scale(scale, scale)
                .post_translate(translate_x, translate_y),
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
        .bind(("127.0.0.1", 3001))?
        .run();

    Ok(server)
}