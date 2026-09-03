# Alpine Weather Route

A static, browser-only weather planner for cycling in the Alps and Dolomites. Upload a GPX route to see 16-day hourly forecasts along it, or explore curated locations from Chamonix to Venice, Slovenia, and Bodensee.

There is no application server: GPX parsing, route sampling, arrival-time estimation, and weather requests all run in the browser. Forecast data comes directly from [Open-Meteo](https://open-meteo.com/) and maps from [OpenStreetMap](https://www.openstreetmap.org/).

## Local development

Because browsers restrict some features when opening files directly, serve the directory with any static server:

```console
python3 -m http.server 8000 --directory static
```

Then open <http://localhost:8000>.

## GitHub Pages

The workflow in `.github/workflows/pages.yml` publishes `static/` whenever `main` is pushed. In the repository settings, select **GitHub Actions** as the Pages source. No API keys or repository secrets are required.

The site uses relative asset paths, so it works both at a user/organization Pages root and under a project path such as `https://owner.github.io/repository/`.

## Limitations

- Uploaded GPX files stay in the browser and are not persisted or sent to this repository.
- Weather requests are sent from the visitor's browser to Open-Meteo.
- Model forecasts simplify complex mountain terrain. Check official warnings and local mountain forecasts before riding.
- Forecast availability is limited to Open-Meteo's current forecast horizon.
