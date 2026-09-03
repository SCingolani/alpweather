# Alpine Weather Route

Upload a GPX cycling route and inspect hourly weather forecasts along it on an interactive map. The application samples the route roughly every 10 km, estimates arrival times from departure and riding speed, and retrieves 16-day hourly forecasts from Open-Meteo.

## Run locally

```console
cargo run
```

Then open <http://127.0.0.1:3000>. With Nix, use `nix develop` for a development shell or `nix run` once the lock file has been generated.

Configuration is through `WEATHER_LISTEN_ADDRESS` (default `127.0.0.1`), `WEATHER_PORT` (default `3000`), `WEATHER_STATIC_DIR` (default `static`), `WEATHER_SAMPLE_KM` (default `10`) and `RUST_LOG`.

## API

`POST /api/trips` accepts multipart fields `gpx`, `departure` (RFC 3339) and `speed_kmh`. It returns the route plus all forecast samples. `GET /api/trips/{id}` returns a previously uploaded trip; trips currently live in process memory. `GET /api/alps` returns forecasts for a curated set of Alpine passes, peaks, valleys, and cycling bases for the no-GPX overview mode.

Forecasts inevitably simplify conditions in complex terrain. Check official warnings and local mountain forecasts before riding. Weather data is attributed to Open-Meteo and its upstream national/model providers; map data is from OpenStreetMap contributors.

## NixOS deployment

Add this flake as an input, import `inputs.alpine-weather-route.nixosModules.default`, then enable the service:

```nix
services.alpine-weather-route = {
  enable = true;
  listenAddress = "0.0.0.0";
  port = 3000;
  openFirewall = true;
};
```
