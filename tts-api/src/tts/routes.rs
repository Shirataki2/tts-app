use crate::{
    auth::token::Token,
    backend::{openjtalk::OpenJTalk, TtsEngine},
    config::Config,
    error::AppError,
    models::users::User,
};
use actix_web::{get, web, HttpResponse, Result};
use sqlx::PgPool;

const OPUS_SAMPLING_RATE: usize = 48000;
const OPUS_MILLIS_PER_FRAME: usize = 20;
const OPUS_FRAME_SIZE: usize = OPUS_SAMPLING_RATE * OPUS_MILLIS_PER_FRAME / 1000;

#[derive(Debug, Deserialize, Default)]
struct TtsGenerateQuery {
    text: String,
    token: String,
    id: i64,
}

#[derive(Debug, Deserialize, Default)]
struct UserQuery {
    token: String,
    id: i64,
}

impl From<UserQuery> for TtsGenerateQuery {
    fn from(query: UserQuery) -> TtsGenerateQuery {
        TtsGenerateQuery {
            text: String::from(""),
            token: query.token,
            id: query.id,
        }
    }
}

#[derive(Serialize, Debug)]
struct OpusDataResponse {
    data: Vec<Vec<u8>>,
}

async fn tts_validate(query: &TtsGenerateQuery, pool: &PgPool) -> Result<User, HttpResponse> {
    if query.text.chars().count() > 200 {
        return Err(
            HttpResponse::BadRequest().body("Text length must be less than 100 characters.")
        );
    }

    let token = Token::new(&query.token);
    let id = query.id;

    match token.verify(pool, id).await {
        Ok(true) => {}
        Ok(false) => return Err(HttpResponse::Unauthorized().body("Invalid token.")),
        Err(AppError::DatabaseError(sqlx::Error::RowNotFound)) => {
            return Err(HttpResponse::NotFound().body("User not found"))
        },
        Err(e) => {
            error!("{:?}", e);
            return Err(HttpResponse::InternalServerError().body("Unexpected Error"));
        }
    }

    let user = match User::get_or_create(pool, id).await {
        Ok(user) => user,
        Err(_) => return Err(HttpResponse::InternalServerError().body("Unexpected Error")),
    };

    let length = query.text.chars().count() as i64;

    if user.character_count + length > user.character_limit {
        return Err(HttpResponse::TooManyRequests().body("Account quota exceeded."));
    }

    if User::use_capability(pool, id, length).await.is_err() {
        return Err(HttpResponse::InternalServerError().body("Unexpected Error"));
    }

    let user = match User::get_or_create(pool, id).await {
        Ok(user) => user,
        Err(_) => return Err(HttpResponse::InternalServerError().body("Unexpected Error")),
    };

    Ok(user)
}

#[get("/user")]
async fn get_user(pool: web::Data<PgPool>, query: web::Query<UserQuery>) -> Result<HttpResponse, HttpResponse> {
    let query = query.into_inner().into();
    let pool = pool.get_ref();

    match tts_validate(&query, pool).await {
        Ok(user) => Ok(HttpResponse::Ok().json(user)),
        Err(err) => Ok(err),
    }
}

#[get("/tts/generate.wav")]
async fn generate_wav(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    query: web::Query<TtsGenerateQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    let pool = pool.get_ref();

    if let Err(e) = tts_validate(&query, pool).await {
        return Ok(e);
    };

    let jtalk_config = config.get_ref().openjtalk.clone();
    let engine = OpenJTalk::from_config(jtalk_config)?;
    let buffer = web::block(move || engine.generate(&query.text)).await;
    let buffer = match buffer {
        Ok(buffer) => buffer,
        Err(err) => {
            error!("{:#?}", err);
            return Ok(HttpResponse::InternalServerError().body("Internal server error"));
        }
    };
    Ok(HttpResponse::Ok().body(buffer))
}

#[get("/tts/generate.opus")]
async fn generate_opus(
    pool: web::Data<PgPool>,
    config: web::Data<Config>,
    query: web::Query<TtsGenerateQuery>,
) -> Result<HttpResponse, AppError> {
    let query = query.into_inner();
    let pool = pool.get_ref();
    if let Err(e) = tts_validate(&query, pool).await {
        return Ok(e);
    };

    let jtalk_config = config.get_ref().openjtalk.clone();
    let engine = OpenJTalk::from_config(jtalk_config)?;
    let buffer = web::block(move || engine.generate_i16(&query.text))
        .await
        .map_err(|_| AppError::SubprocessError())?;

    let mut encoder = opus::Encoder::new(
        OPUS_SAMPLING_RATE as u32,
        opus::Channels::Mono,
        opus::Application::Audio,
    )?;

    let chunks = buffer
        .chunks(OPUS_FRAME_SIZE)
        .map(|chunk| {
            let v = Vec::from(chunk);
            let mut buf = vec![0u8; 256];
            let len = match encoder.encode(&v, &mut buf) {
                Ok(len) => len,
                Err(e) => return Err(e),
            };
            Ok(Vec::from(&buf[..len]))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HttpResponse::Ok().json::<OpusDataResponse>(OpusDataResponse { data: chunks }))
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
    cfg.service(generate_wav);
    cfg.service(generate_opus);
}
