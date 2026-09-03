use anyhow::{bail, Context};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::model::{Coordinate, ForecastHour};

const HOURLY: &str = "temperature_2m,apparent_temperature,dew_point_2m,precipitation,precipitation_probability,rain,showers,snowfall,snow_depth,weather_code,cloud_cover,visibility,wind_speed_10m,wind_gusts_10m,wind_direction_10m,freezing_level_height,cape,uv_index,relative_humidity_2m,surface_pressure,pressure_msl,is_day,sunshine_duration,direct_radiation";

#[derive(Debug)]
pub struct PointWeather {
    pub elevation_m: Option<f64>,
    pub timezone: String,
    pub hours: Vec<ForecastHour>,
}

#[derive(Debug, Deserialize)]
struct RawForecast {
    elevation: Option<f64>,
    timezone: String,
    hourly: RawHourly,
}

#[derive(Debug, Deserialize)]
struct RawHourly {
    time: Vec<String>,
    temperature_2m: Option<Vec<Option<f64>>>,
    apparent_temperature: Option<Vec<Option<f64>>>,
    dew_point_2m: Option<Vec<Option<f64>>>,
    precipitation: Option<Vec<Option<f64>>>,
    precipitation_probability: Option<Vec<Option<f64>>>,
    rain: Option<Vec<Option<f64>>>,
    showers: Option<Vec<Option<f64>>>,
    snowfall: Option<Vec<Option<f64>>>,
    snow_depth: Option<Vec<Option<f64>>>,
    weather_code: Option<Vec<Option<i32>>>,
    cloud_cover: Option<Vec<Option<f64>>>,
    visibility: Option<Vec<Option<f64>>>,
    wind_speed_10m: Option<Vec<Option<f64>>>,
    wind_gusts_10m: Option<Vec<Option<f64>>>,
    wind_direction_10m: Option<Vec<Option<f64>>>,
    freezing_level_height: Option<Vec<Option<f64>>>,
    cape: Option<Vec<Option<f64>>>,
    uv_index: Option<Vec<Option<f64>>>,
    relative_humidity_2m: Option<Vec<Option<f64>>>,
    surface_pressure: Option<Vec<Option<f64>>>,
    pressure_msl: Option<Vec<Option<f64>>>,
    is_day: Option<Vec<Option<i32>>>,
    sunshine_duration: Option<Vec<Option<f64>>>,
    direct_radiation: Option<Vec<Option<f64>>>,
}

pub async fn fetch_forecasts(
    client: &Client,
    coordinates: &[Coordinate],
) -> anyhow::Result<Vec<PointWeather>> {
    let mut all = Vec::with_capacity(coordinates.len());
    for chunk in coordinates.chunks(40) {
        let latitudes = chunk
            .iter()
            .map(|p| p.lat.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let longitudes = chunk
            .iter()
            .map(|p| p.lon.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let elevations = chunk
            .iter()
            .map(|p| {
                p.elevation_m
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "nan".into())
            })
            .collect::<Vec<_>>()
            .join(",");
        let response = client
            .get("https://api.open-meteo.com/v1/forecast")
            .query(&[
                ("latitude", latitudes.as_str()),
                ("longitude", longitudes.as_str()),
                ("elevation", elevations.as_str()),
                ("hourly", HOURLY),
                ("forecast_days", "16"),
                ("timezone", "GMT"),
                ("wind_speed_unit", "kmh"),
            ])
            .send()
            .await
            .context("could not reach Open-Meteo")?
            .error_for_status()
            .context("Open-Meteo rejected the forecast request")?;
        let value: serde_json::Value =
            response.json().await.context("invalid forecast response")?;
        let raws: Vec<RawForecast> = if value.is_array() {
            serde_json::from_value(value)?
        } else {
            vec![serde_json::from_value(value)?]
        };
        if raws.len() != chunk.len() {
            bail!(
                "weather provider returned {} locations for {} requested",
                raws.len(),
                chunk.len()
            );
        }
        all.extend(raws.into_iter().map(convert));
    }
    Ok(all)
}

fn at<T: Copy>(values: &Option<Vec<Option<T>>>, index: usize) -> Option<T> {
    values.as_ref()?.get(index).copied().flatten()
}

fn convert(raw: RawForecast) -> PointWeather {
    let hours = raw
        .hourly
        .time
        .iter()
        .enumerate()
        .filter_map(|(i, time)| {
            let time = DateTime::parse_from_rfc3339(&format!("{time}:00Z"))
                .ok()?
                .with_timezone(&Utc);
            Some(ForecastHour {
                time,
                temperature_c: at(&raw.hourly.temperature_2m, i),
                apparent_temperature_c: at(&raw.hourly.apparent_temperature, i),
                dew_point_c: at(&raw.hourly.dew_point_2m, i),
                precipitation_mm: at(&raw.hourly.precipitation, i),
                precipitation_probability_pct: at(&raw.hourly.precipitation_probability, i),
                rain_mm: at(&raw.hourly.rain, i),
                showers_mm: at(&raw.hourly.showers, i),
                snowfall_cm: at(&raw.hourly.snowfall, i),
                snow_depth_m: at(&raw.hourly.snow_depth, i),
                weather_code: at(&raw.hourly.weather_code, i),
                cloud_cover_pct: at(&raw.hourly.cloud_cover, i),
                visibility_m: at(&raw.hourly.visibility, i),
                wind_speed_kmh: at(&raw.hourly.wind_speed_10m, i),
                wind_gust_kmh: at(&raw.hourly.wind_gusts_10m, i),
                wind_direction_deg: at(&raw.hourly.wind_direction_10m, i),
                freezing_level_m: at(&raw.hourly.freezing_level_height, i),
                cape_jkg: at(&raw.hourly.cape, i),
                uv_index: at(&raw.hourly.uv_index, i),
                relative_humidity_pct: at(&raw.hourly.relative_humidity_2m, i),
                surface_pressure_hpa: at(&raw.hourly.surface_pressure, i),
                pressure_msl_hpa: at(&raw.hourly.pressure_msl, i),
                is_day: at(&raw.hourly.is_day, i),
                sunshine_duration_s: at(&raw.hourly.sunshine_duration, i),
                direct_radiation_wm2: at(&raw.hourly.direct_radiation, i),
            })
        })
        .collect();
    PointWeather {
        elevation_m: raw.elevation,
        timezone: raw.timezone,
        hours,
    }
}
