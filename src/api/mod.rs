mod cors;

use crate::structs::{Animator, ColorMode};
use rocket::serde::{json::Json, Serialize};
use rocket::{fs::NamedFile, Build, Rocket, State};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

struct AppState {
    color_mode: Arc<Mutex<ColorMode>>,
    animator: Arc<Mutex<Animator>>,
}
#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
struct Message {
    status: String,
    r#type: String,
    data: String,
}

#[post("/color_mode", data = "<mode>")]
async fn set_color_mode(state: &State<AppState>, mode: String) -> Json<Message> {
    state
        .color_mode
        .lock()
        .expect("Could not take the lock on `color_mode`")
        .set_color_mode(mode);
    Json(Message {
        status: "success".to_string(),
        r#type: "color_mode".to_string(),
        data: state
            .color_mode
            .lock()
            .expect("Could not take the lock on `color_mode`")
            .mode
            .to_string(),
    })
}
#[get("/color_mode")]
async fn get_color_mode(state: &State<AppState>) -> Json<Message> {
    Json(Message {
        status: "success".to_string(),
        r#type: "color_mode".to_string(),
        data: state
            .color_mode
            .lock()
            .expect("Could not take the lock on `color_mode`")
            .mode
            .to_string(),
    })
}
#[post("/animation", data = "<animation>")]
async fn set_animation(state: &State<AppState>, animation: String) -> Json<Message> {
    state
        .animator
        .lock()
        .expect("Could not take the lock on `animator`")
        .set_animation(animation);
    Json(Message {
        status: "success".to_string(),
        r#type: "animation".to_string(),
        data: state
            .animator
            .lock()
            .expect("Could not take the lock on `animator`")
            .animation
            .to_string(),
    })
}
#[get("/animation")]
async fn get_animation(state: &State<AppState>) -> Json<Message> {
    Json(Message {
        status: "success".to_string(),
        r#type: "animation".to_string(),
        data: state
            .animator
            .lock()
            .expect("Could not take the lock on `animator`")
            .animation
            .to_string(),
    })
}
#[post("/brightness", data = "<brightness>")]
async fn set_brightness(_state: &State<AppState>, brightness: String) -> Json<Message> {
    if let Ok(_brightness) = brightness.parse::<u8>() {
        Json(Message {
            status: "error".to_string(),
            r#type: "brightness".to_string(),
            data: "NOT_IMPLEMENTED".to_string(),
        })
    } else {
        Json(Message {
            status: "error".to_string(),
            r#type: "brightness".to_string(),
            data: "Could not parse brightness".to_string(),
        })
    }
}
#[get("/brightness")]
async fn get_brightness(_state: &State<AppState>) -> Json<Message> {
    Json(Message {
        status: "error".to_string(),
        r#type: "brightness".to_string(),
        data: "NOT_IMPLEMENTED".to_string(),
    })
}

#[get("/static/<file..>")]
async fn files(file: PathBuf) -> NamedFile {
    NamedFile::open(Path::new("/home/pi/web/build/static").join(file))
        .await
        .expect("Could not open file")
}
#[get("/")]
async fn index() -> NamedFile {
    NamedFile::open("/home/pi/web/build/index.html")
        .await
        .expect("Could not open file")
}
pub fn main(color_mode: &Arc<Mutex<ColorMode>>, animator: &Arc<Mutex<Animator>>) -> Rocket<Build> {
    rocket::build()
        .attach(cors::CORS)
        .manage(AppState {
            color_mode: color_mode.clone(),
            animator: animator.clone(),
        })
        .mount("/", routes![files, index])
        .mount(
            "/api",
            routes![
                get_color_mode,
                set_color_mode,
                get_animation,
                set_animation,
                get_brightness,
                set_brightness
            ],
        )
}
