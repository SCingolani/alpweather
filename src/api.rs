use std::{collections::HashMap, path::PathBuf, sync::Arc};

use axum::{
    extract::{DefaultBodyLimit, Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Duration, Utc};
use reqwest::Client;
use serde::Serialize;
use tokio::sync::RwLock;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use uuid::Uuid;

use crate::{
    model::{ForecastPoint, Trip},
    route, weather,
};

pub struct AppState {
    pub trips: RwLock<HashMap<Uuid, Arc<Trip>>>,
    pub client: Client,
    pub sample_km: f64,
}

#[derive(Debug)]
struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            error: String,
        }
        (self.0, Json(ErrorBody { error: self.1 })).into_response()
    }
}

pub fn router(state: Arc<AppState>, static_dir: PathBuf) -> Router {
    let index = static_dir.join("index.html");
    Router::new()
        .route(
            "/api/health",
            get(|| async { Json(serde_json::json!({"status":"ok"})) }),
        )
        .route("/api/trips", post(create_trip))
        .route("/api/trips/{id}", get(get_trip))
        .route("/api/alps", get(get_alps))
        .nest_service("/static", ServeDir::new(static_dir))
        .fallback_service(ServeFile::new(index))
        .layer(DefaultBodyLimit::max(20 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn get_trip(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Arc<Trip>>, ApiError> {
    state
        .trips
        .read()
        .await
        .get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| ApiError(StatusCode::NOT_FOUND, "trip not found".into()))
}

async fn create_trip(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Arc<Trip>>), ApiError> {
    let mut gpx = None;
    let mut filename = "route".to_string();
    let mut departure = Utc::now();
    let mut speed_kmh = 18.0;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| bad_request(e.to_string()))?
    {
        match field.name() {
            Some("gpx") => {
                if let Some(name) = field.file_name() {
                    filename = name.trim_end_matches(".gpx").to_string();
                }
                gpx = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|e| bad_request(e.to_string()))?
                        .to_vec(),
                );
            }
            Some("departure") | Some("departure_datetime") => {
                let text = field.text().await.map_err(|e| bad_request(e.to_string()))?;
                departure = DateTime::parse_from_rfc3339(&text)
                    .map_err(|_| {
                        bad_request("departure must be an ISO-8601 timestamp with timezone")
                    })?
                    .with_timezone(&Utc);
            }
            Some("speed_kmh") => {
                speed_kmh = field
                    .text()
                    .await
                    .map_err(|e| bad_request(e.to_string()))?
                    .parse()
                    .map_err(|_| bad_request("speed_kmh must be a number"))?;
            }
            _ => {}
        }
    }
    if !(3.0..=80.0).contains(&speed_kmh) {
        return Err(bad_request("speed_kmh must be between 3 and 80"));
    }
    let parsed = route::parse_gpx(
        &gpx.ok_or_else(|| bad_request("missing gpx file"))?,
        &filename,
    )
    .map_err(|e| bad_request(e.to_string()))?;
    let samples = route::sample_route(&parsed.points, state.sample_km);
    if samples.len() > 200 {
        return Err(bad_request(
            "route is too long or sampling interval is too small (maximum 200 forecast points)",
        ));
    }
    let coordinates = samples.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>();
    let weather = weather::fetch_forecasts(&state.client, &coordinates)
        .await
        .map_err(|e| ApiError(StatusCode::BAD_GATEWAY, e.to_string()))?;
    let forecasts = samples
        .into_iter()
        .zip(weather)
        .map(|((coordinate, distance_km), weather)| ForecastPoint {
            name: None,
            coordinate,
            distance_km,
            eta: departure + Duration::milliseconds((distance_km / speed_kmh * 3_600_000.0) as i64),
            provider_elevation_m: weather.elevation_m,
            timezone: weather.timezone,
            hours: weather.hours,
        })
        .collect();
    let trip = Arc::new(Trip {
        id: Uuid::new_v4(),
        name: parsed.name,
        created_at: Utc::now(),
        departure,
        speed_kmh,
        distance_km: parsed.distance_km,
        ascent_m: parsed.ascent_m,
        route: parsed.points,
        forecasts,
        attribution: "Weather data by Open-Meteo.com; map © OpenStreetMap contributors".into(),
    });
    state.trips.write().await.insert(trip.id, trip.clone());
    Ok((StatusCode::CREATED, Json(trip)))
}

async fn get_alps(State(state): State<Arc<AppState>>) -> Result<Json<Arc<Trip>>, ApiError> {
    const PLACES: &[(&str, f64, f64, f64)] = &[
        ("Chamonix", 45.9237, 6.8694, 1035.0),
        ("Col du Galibier", 45.0641, 6.4078, 2642.0),
        ("Zermatt", 46.0207, 7.7491, 1608.0),
        ("Furka Pass", 46.5729, 8.4152, 2429.0),
        ("Stelvio Pass", 46.5286, 10.4532, 2757.0),
        ("St. Moritz", 46.4908, 9.8355, 1822.0),
        ("Innsbruck", 47.2692, 11.4041, 574.0),
        ("Grossglockner Pass", 47.0745, 12.8425, 2504.0),
        ("Bolzano", 46.4983, 11.3548, 262.0),
        ("Merano", 46.6713, 11.1594, 325.0),
        ("Reschen Pass", 46.8340, 10.5061, 1504.0),
        ("Brenner Pass", 47.0036, 11.5064, 1370.0),
        ("Val Gardena", 46.5572, 11.7180, 1563.0),
        ("Canazei", 46.4769, 11.7712, 1465.0),
        ("Arabba", 46.4972, 11.8758, 1602.0),
        ("Sella Pass", 46.5088, 11.7578, 2240.0),
        ("Passo Giau", 46.4831, 12.0520, 2236.0),
        ("Alleghe", 46.4075, 12.0231, 979.0),
        ("Cortina d'Ampezzo", 46.5405, 12.1357, 1224.0),
        ("Tre Cime", 46.6187, 12.3028, 2320.0),
        ("Belluno", 46.1400, 12.2175, 390.0),
        ("Vittorio Veneto", 45.9803, 12.2996, 138.0),
        ("Treviso", 45.6669, 12.2430, 15.0),
        ("Venice", 45.4408, 12.3155, 2.0),
        ("Bassano del Grappa", 45.7666, 11.7340, 129.0),
        ("Lake Garda — Riva", 45.8858, 10.8412, 70.0),
        ("Monte Zoncolan", 46.5000, 12.9200, 1730.0),
        ("Bled", 46.3692, 14.1136, 501.0),
        ("Vršič Pass", 46.4352, 13.7441, 1611.0),
        ("Landeck", 47.1399, 10.5659, 817.0),
        ("Arlberg Pass", 47.1292, 10.2110, 1793.0),
        ("Bregenz", 47.5031, 9.7471, 400.0),
        ("Lindau", 47.5460, 9.6844, 401.0),
        ("Konstanz", 47.6779, 9.1732, 405.0),
    ];
    let coordinates = PLACES
        .iter()
        .map(|(_, lat, lon, elevation)| crate::model::Coordinate {
            lat: *lat,
            lon: *lon,
            elevation_m: Some(*elevation),
        })
        .collect::<Vec<_>>();
    let weather = weather::fetch_forecasts(&state.client, &coordinates)
        .await
        .map_err(|e| ApiError(StatusCode::BAD_GATEWAY, e.to_string()))?;
    let now = Utc::now();
    let forecasts = PLACES
        .iter()
        .zip(coordinates)
        .zip(weather)
        .map(|(((name, _, _, _), coordinate), weather)| ForecastPoint {
            name: Some((*name).to_string()),
            coordinate,
            distance_km: 0.0,
            eta: now,
            provider_elevation_m: weather.elevation_m,
            timezone: weather.timezone,
            hours: weather.hours,
        })
        .collect();
    Ok(Json(Arc::new(Trip {
        id: Uuid::new_v4(),
        name: "Alpine overview".into(),
        created_at: now,
        departure: now,
        speed_kmh: 0.0,
        distance_km: 0.0,
        ascent_m: 0.0,
        route: Vec::new(),
        forecasts,
        attribution: "Weather data by Open-Meteo.com; map © OpenStreetMap contributors".into(),
    })))
}

fn bad_request(message: impl Into<String>) -> ApiError {
    ApiError(StatusCode::BAD_REQUEST, message.into())
}
