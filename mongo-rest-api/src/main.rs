use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use mongodb::{bson::doc, options::IndexOptions, Client, Collection, IndexModel};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClassicModule {
    pub module_id: String,
    pub module_name: String,
    pub is_deleted: bool,
    pub last_updated_at: String,
}

const DB_NAME: &str = "classic";
const COLL_NAME: &str = "classicmodules";

#[get("/hello/{name}")]
async fn greet(name: web::Path<String>) -> impl Responder {
    format!("Hello {name}!")
}

#[post("/add_module")]
async fn add_module(client: web::Data<Client>, form: web::Json<ClassicModule>) -> HttpResponse {
    println!("{form:?}");
    let collection = client.database(DB_NAME).collection(COLL_NAME);
    let result = collection.insert_one(form.into_inner(), None).await;
    match result {
        Ok(_) => HttpResponse::Ok().body("classic module added"),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

#[get("/module/{module_id}")]
async fn get_module(client: web::Data<Client>, module_id: web::Path<String>) -> HttpResponse {
    let module_id = module_id.into_inner();
    let collection: Collection<ClassicModule> = client.database(DB_NAME).collection(COLL_NAME);
    match collection
        .find_one(doc! { "module_id": &module_id }, None)
        .await
    {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().body(format!("No module found with id {module_id}")),
        Err(err) => HttpResponse::InternalServerError().body(err.to_string()),
    }
}

async fn create_username_index(client: &Client) {
    let options = IndexOptions::builder().unique(true).build();
    let model = IndexModel::builder()
        .keys(doc! { "module_id": 1 })
        .options(options)
        .build();
    client
        .database(DB_NAME)
        .collection::<ClassicModule>(COLL_NAME)
        .create_index(model, None)
        .await
        .expect("creating an index should succeed");
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mongo_uri = "mongodb://localhost:27017";

    let mongo_client = Client::with_uri_str(mongo_uri)
        .await
        .expect("failed to connect");

    create_username_index(&mongo_client).await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(mongo_client.clone()))
            .service(greet)
            .service(add_module)
            .service(get_module)
    })
    .bind(("localhost", 8080))?
    .run()
    .await
}
