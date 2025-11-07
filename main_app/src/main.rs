mod migrations;

mod my_random;
mod models;
mod shazam;
mod views;
mod forms;
mod download_helpers;
mod handlers;

// mod utils;

use askama::Template;
use cot::auth::db::DatabaseUserApp;
use cot::cli::CliMetadata;
use cot::db::migrations::SyncDynMigration;
use cot::html::Html;
use cot::middleware::{AuthMiddleware, LiveReloadMiddleware, SessionMiddleware};
use cot::project::{MiddlewareContext, RegisterAppsContext, RootHandler, RootHandlerBuilder};
use cot::request::extractors::StaticFiles;
use cot::router::{Route, Router};
use cot::static_files::{StaticFile, StaticFilesMiddleware};
use cot::session::db::SessionApp;
use cot::{App, AppBuilder, Project, static_files};
use cot::db::{model, Auto, LimitedString};
use cot::request::extractors::RequestDb;
use cot::db::Model;
use cot::db::query;


#[model]
pub struct Link {
    #[model(primary_key)]
    id: Auto<i64>,
    url: String,
}

impl Link{
    fn new(url: &str)->Link{
        Link{
            id: Auto::default(),
            url: url.to_string()
        }
    }
}
#[derive(Debug, Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    static_files: StaticFiles,
}

async fn index(static_files: StaticFiles, RequestDb(mut db): RequestDb) -> cot::Result<Html> {
    let index_template = IndexTemplate { static_files };
    let rendered = index_template.render()?;

    let mut link = Link::new("abc");
    link.save(&mut db);

    let link = query!(Link, $url != String::from("sfasdf"))
        .all(&mut db)
        .await?;

    println!("LEN S: {}", link.len());
    Ok(Html::new(rendered))
}

// async fn myview(request: Request,  RequestDb(mut db): RequestDb) -> () {
    // let index_template = IndexTemplate { static_files };
    // let rendered = index_template.render()?;

    // Ok(Html::new(rendered))
// }

struct MainAppApp;

impl App for MainAppApp {
    fn name(&self) -> &'static str {
        env!("CARGO_CRATE_NAME")
    }

    fn migrations(&self) -> Vec<Box<SyncDynMigration>> {
        cot::db::migrations::wrap_migrations(migrations::MIGRATIONS)
    }

    fn router(&self) -> Router {
        Router::with_urls([
            Route::with_handler_and_name("", index, "index"),
            Route::with_handler_and_name(
                "upload/",
                views::upload_view,
                "upload-view"
            ),
            Route::with_handler_and_name(
                "search/",
                views::search_view,
                "search-view"
            ),

        ])
    }

    fn static_files(&self) -> Vec<StaticFile> {
        static_files!("css/main.css")
    }
}

struct MainAppProject;

impl Project for MainAppProject {
    fn cli_metadata(&self) -> CliMetadata {
        cot::cli::metadata!()
    }

    fn register_apps(&self, apps: &mut AppBuilder, _context: &RegisterAppsContext) {
        apps.register_with_views(MainAppApp, "/",);
        apps.register(DatabaseUserApp::new());
        apps.register(SessionApp::new());
    }

    fn middlewares(
        &self,
        handler: RootHandlerBuilder,
        context: &MiddlewareContext,
    ) -> RootHandler {
        handler
            .middleware(StaticFilesMiddleware::from_context(context))
            .middleware(AuthMiddleware::new())
            .middleware(SessionMiddleware::from_context(context))
            .middleware(LiveReloadMiddleware::from_context(context))
            .build()
    }
}

#[cot::main]
fn main() -> impl Project {
    MainAppProject
}


