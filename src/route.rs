use std::io::Cursor;

use anyhow::{bail, Context};
use gpx::{read, Gpx};

use crate::model::Coordinate;

pub struct ParsedRoute {
    pub name: String,
    pub points: Vec<Coordinate>,
    pub distance_km: f64,
    pub ascent_m: f64,
}

pub fn parse_gpx(bytes: &[u8], fallback_name: &str) -> anyhow::Result<ParsedRoute> {
    let gpx: Gpx = read(Cursor::new(bytes)).context("invalid GPX document")?;
    let mut points = Vec::new();
    for track in &gpx.tracks {
        for segment in &track.segments {
            for p in &segment.points {
                points.push(Coordinate {
                    lat: p.point().y(),
                    lon: p.point().x(),
                    elevation_m: p.elevation,
                });
            }
        }
    }
    if points.is_empty() {
        for route in &gpx.routes {
            for p in &route.points {
                points.push(Coordinate {
                    lat: p.point().y(),
                    lon: p.point().x(),
                    elevation_m: p.elevation,
                });
            }
        }
    }
    if points.len() < 2 {
        bail!("GPX must contain a track or route with at least two points");
    }
    let distance_km = points.windows(2).map(|w| haversine_km(&w[0], &w[1])).sum();
    let ascent_m = points
        .windows(2)
        .filter_map(|w| Some((w[1].elevation_m? - w[0].elevation_m?).max(0.0)))
        .sum();
    let name = gpx
        .tracks
        .first()
        .and_then(|t| t.name.clone())
        .or_else(|| gpx.routes.first().and_then(|r| r.name.clone()))
        .unwrap_or_else(|| fallback_name.to_string());
    Ok(ParsedRoute {
        name,
        points,
        distance_km,
        ascent_m,
    })
}

pub fn sample_route(points: &[Coordinate], spacing_km: f64) -> Vec<(Coordinate, f64)> {
    let total: f64 = points.windows(2).map(|w| haversine_km(&w[0], &w[1])).sum();
    let spacing = spacing_km.max(1.0);
    let mut targets: Vec<f64> = (0..)
        .map(|i| i as f64 * spacing)
        .take_while(|d| *d < total)
        .collect();
    targets.push(total);
    let mut result = Vec::with_capacity(targets.len());
    let mut segment_start = 0.0;
    let mut target_index = 0;
    for pair in points.windows(2) {
        let length = haversine_km(&pair[0], &pair[1]);
        while target_index < targets.len() && targets[target_index] <= segment_start + length + 1e-9
        {
            let fraction = if length > 0.0 {
                (targets[target_index] - segment_start) / length
            } else {
                0.0
            };
            result.push((
                interpolate(&pair[0], &pair[1], fraction.clamp(0.0, 1.0)),
                targets[target_index],
            ));
            target_index += 1;
        }
        segment_start += length;
    }
    result
}

fn interpolate(a: &Coordinate, b: &Coordinate, t: f64) -> Coordinate {
    Coordinate {
        lat: a.lat + (b.lat - a.lat) * t,
        lon: a.lon + (b.lon - a.lon) * t,
        elevation_m: match (a.elevation_m, b.elevation_m) {
            (Some(x), Some(y)) => Some(x + (y - x) * t),
            _ => None,
        },
    }
}

fn haversine_km(a: &Coordinate, b: &Coordinate) -> f64 {
    let (lat1, lat2) = (a.lat.to_radians(), b.lat.to_radians());
    let dlat = lat2 - lat1;
    let dlon = (b.lon - a.lon).to_radians();
    let h = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    6371.0088 * 2.0 * h.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_endpoints_and_intermediate_points() {
        let points = vec![
            Coordinate {
                lat: 46.0,
                lon: 11.0,
                elevation_m: Some(1000.0),
            },
            Coordinate {
                lat: 46.0,
                lon: 11.5,
                elevation_m: Some(2000.0),
            },
        ];
        let samples = sample_route(&points, 10.0);
        assert_eq!(samples.first().unwrap().1, 0.0);
        assert!((samples.last().unwrap().0.lon - 11.5).abs() < 1e-8);
        assert!(samples.len() >= 4);
    }
}
