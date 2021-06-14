use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{EmptyMutation, EmptySubscription, Object, Schema};
use rocket::fs::{relative, FileServer};
use rocket::response::content;
use rocket::{State, get, info_, post, routes};
use rocket_dyn_templates::Template;
use serde_json::json;

mod async_graphql_rocket;
use async_graphql_rocket::{BatchRequest, Query, Response};

mod encoder;
use encoder::DflateEncoder;

#[get("/?<all>")]
fn index(all: Option<u8>) -> Template {
    let context = json!({
        "hello": "Привет",
        "all": all.unwrap_or(0),
        "people": [
            "Yehuda Katz",
            "Alan Johnson",
            "Charles Jolley",
          ],
    });
    info_!("Context: {}", context);
    Template::render("index", &context)
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Returns the sum of a and b
    async fn add(&self, a: i32, b: i32) -> i32 {
        a + b
    }
}

type StarWarsSchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[get("/gq")]
fn graphql_playground() -> content::Html<String> {
    content::Html(playground_source(GraphQLPlaygroundConfig::new("/graphql")))
}

#[get("/graphql?<query..>")]
async fn graphql_query(schema: &State<StarWarsSchema>, query: Query) -> Response {
    query.execute(schema).await
}

#[post("/graphql", data = "<request>")]
async fn graphql_request(schema: &State<StarWarsSchema>, request: BatchRequest) -> Response {
    request.execute(schema).await
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).finish();

    rocket::build()
        .manage(schema)
        .mount(
            "/",
            routes![index, graphql_playground, graphql_query, graphql_request],
        )
        .mount("/", FileServer::from(relative!("static")))
        .attach(Template::fairing())
        .attach(DflateEncoder)
        .launch()
        .await
}
