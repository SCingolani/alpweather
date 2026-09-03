use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
    pub elevation_m: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastHour {
    pub time: DateTime<Utc>,
    pub temperature_c: Option<f64>,
    pub apparent_temperature_c: Option<f64>,
    pub dew_point_c: Option<f64>,
    pub precipitation_mm: Option<f64>,
    pub precipitation_probability_pct: Option<f64>,
    pub rain_mm: Option<f64>,
    pub showers_mm: Option<f64>,
    pub snowfall_cm: Option<f64>,
    pub snow_depth_m: Option<f64>,
    pub weather_code: Option<i32>,
    pub cloud_cover_pct: Option<f64>,
    pub visibility_m: Option<f64>,
    pub wind_speed_kmh: Option<f64>,
    pub wind_gust_kmh: Option<f64>,
    pub wind_direction_deg: Option<f64>,
    pub freezing_level_m: Option<f64>,
    pub cape_jkg: Option<f64>,
    pub uv_index: Option<f64>,
    pub relative_humidity_pct: Option<f64>,
    pub surface_pressure_hpa: Option<f64>,
    pub pressure_msl_hpa: Option<f64>,
    pub is_day: Option<i32>,
    pub sunshine_duration_s: Option<f64>,
    pub direct_radiation_wm2: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    pub name: Option<String>,
    pub coordinate: Coordinate,
    pub distance_km: f64,
    pub eta: DateTime<Utc>,
    pub provider_elevation_m: Option<f64>,
    pub timezone: String,
    pub hours: Vec<ForecastHour>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub departure: DateTime<Utc>,
    pub speed_kmh: f64,
    pub distance_km: f64,
    pub ascent_m: f64,
    pub route: Vec<Coordinate>,
    pub forecasts: Vec<ForecastPoint>,
    pub attribution: String,
}
