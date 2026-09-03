const $ = id => document.getElementById(id);
const map = L.map('map').setView([46.5, 11.8], 7);
L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', { attribution: '&copy; OpenStreetMap contributors', maxZoom: 18 }).addTo(map);
let routeLayer, markerLayer = L.layerGroup().addTo(map), trip, forecastTimes = [];
const weatherNames = {0:'Clear',1:'Mostly clear',2:'Partly cloudy',3:'Overcast',45:'Fog',48:'Rime fog',51:'Light drizzle',53:'Drizzle',55:'Heavy drizzle',56:'Freezing drizzle',57:'Heavy freezing drizzle',61:'Light rain',63:'Rain',65:'Heavy rain',66:'Freezing rain',67:'Heavy freezing rain',71:'Light snow',73:'Snow',75:'Heavy snow',77:'Snow grains',80:'Rain showers',81:'Rain showers',82:'Heavy showers',85:'Snow showers',86:'Heavy snow showers',95:'Thunderstorm',96:'Thunderstorm with hail',99:'Severe hailstorm'};
const esc = value => { const el = document.createElement('div'); el.textContent = value == null ? '' : value; return el.innerHTML; };
const date = value => { const d = new Date(value); return Number.isNaN(d.getTime()) ? null : d; };
const localInputDate = d => new Date(d.getTime() - d.getTimezoneOffset()*60000).toISOString().slice(0,16);
const fmt = (n, digits=0) => n == null ? '—' : Number(n).toFixed(digits);
function nearestHour(point, when) { return point.hours?.reduce((best, h) => !best || Math.abs(date(h.time)-when) < Math.abs(date(best.time)-when) ? h : best, null); }
function category(h) { const code = h?.weather_code; if (code >= 95) return 'storm'; if ((code >= 71 && code <= 77) || (code >= 85 && code <= 86) || h?.temperature_c <= 0) return 'snow'; if ((code >= 51 && code <= 67) || (code >= 80 && code <= 82) || h?.precipitation_mm > .2) return 'rain'; return 'dry'; }
function weatherIcon(h) {
  const code = h?.weather_code;
  if (code >= 95) return '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 16a4 4 0 0 1 .7-7.9A5.5 5.5 0 0 1 18.4 10 3 3 0 0 1 18 16H7Z"/><path class="weather-accent" d="m13 15-3 5h3l-1 3 5-6h-3l1-2Z"/></svg>';
  if ((code >= 71 && code <= 77) || (code >= 85 && code <= 86)) return '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 14a4 4 0 0 1 .7-7.9A5.5 5.5 0 0 1 18.4 8 3 3 0 0 1 18 14H7Z"/><path class="weather-accent" d="M8 17v4m-2-2h4m5-2v4m-2-2h4"/></svg>';
  if ((code >= 51 && code <= 67) || (code >= 80 && code <= 82)) return '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 14a4 4 0 0 1 .7-7.9A5.5 5.5 0 0 1 18.4 8 3 3 0 0 1 18 14H7Z"/><path class="weather-accent" d="m8 17-1 3m6-3-1 3m6-3-1 3"/></svg>';
  if (code === 45 || code === 48) return '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 13a4 4 0 0 1 .7-7.9A5.5 5.5 0 0 1 18.4 7 3 3 0 0 1 18 13H7Z"/><path class="weather-accent" d="M5 17h14M7 21h10"/></svg>';
  if (code >= 2) return '<svg viewBox="0 0 24 24" aria-hidden="true"><circle class="weather-accent" cx="8" cy="8" r="4"/><path d="M7 18a4 4 0 0 1 .7-7.9A5.5 5.5 0 0 1 18.4 12 3 3 0 0 1 18 18H7Z"/></svg>';
  if (h?.is_day === 0) return '<svg viewBox="0 0 24 24" aria-hidden="true"><path d="M17.5 15.5A7 7 0 0 1 9 6.5a7 7 0 1 0 8.5 9Z"/></svg>';
  return '<svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v3m0 14v3M2 12h3m14 0h3M5 5l2 2m10 10 2 2M19 5l-2 2M7 17l-2 2"/></svg>';
}
function markerIcon(h) {
  const temperature = h?.temperature_c == null ? '—' : `${Math.round(h.temperature_c)}°`;
  return L.divIcon({ className:'weather-marker-shell', html:`<div class="weather-marker ${category(h)}">${weatherIcon(h)}<span>${temperature}</span></div>`, iconSize:[50,30], iconAnchor:[25,15], popupAnchor:[0,-14] });
}
function popup(point, h, selected) {
  const routeElevation = point.coordinate.elevation_m, gridElevation = point.provider_elevation_m;
  const location = point.name ? esc(point.name) : `Km ${fmt(point.distance_km,1)}`;
  const timing = point.name ? '' : `<br><small>Estimated arrival: ${date(point.eta)?.toLocaleString() || '—'}</small>`;
  return `<div class="weather-popup"><strong>${location} · ${selected.toLocaleString([], {weekday:'short',hour:'2-digit',minute:'2-digit'})}</strong><br><b>${esc(weatherNames[h?.weather_code] || 'No data')}</b><br>Temperature: ${fmt(h?.temperature_c,1)} °C (feels ${fmt(h?.apparent_temperature_c,1)} °C)<br>Rain/snow: ${fmt(h?.precipitation_mm,1)} mm / ${fmt(h?.snowfall_cm,1)} cm (${fmt(h?.precipitation_probability_pct)}%)<br>Wind: ${fmt(h?.wind_speed_kmh)} km/h, gusts ${fmt(h?.wind_gust_kmh)} km/h<br>Visibility: ${h?.visibility_m == null?'—':fmt(h.visibility_m/1000,1)} km<br>Cloud / humidity: ${fmt(h?.cloud_cover_pct)}% / ${fmt(h?.relative_humidity_pct)}%<br>Freezing level: ${fmt(h?.freezing_level_m)} m<br>Elevation: ${fmt(routeElevation)} m${gridElevation == null?'':` (forecast ${fmt(gridElevation)} m)`}${timing}</div>`;
}
function renderAt(index) {
  if (!trip) return; const selected = forecastTimes[index] || date(trip.departure); markerLayer.clearLayers();
  trip.forecasts.forEach(point => { const h=nearestHour(point, selected); L.marker([point.coordinate.lat,point.coordinate.lon],{icon:markerIcon(h)}).bindPopup(popup(point,h,selected)).addTo(markerLayer); });
  $('time-label').textContent = selected?.toLocaleString([], {weekday:'short',month:'short',day:'numeric',hour:'2-digit',minute:'2-digit'}) || '—';
}
function render(raw) {
  trip=raw; $('empty-state').classList.add('hidden'); if(routeLayer) routeLayer.remove();
  const coords=trip.route.map(p=>[p.lat,p.lon]);
  if (coords.length > 1) { routeLayer=L.polyline(coords,{color:'#173f34',weight:4,opacity:.9}).addTo(map); map.fitBounds(routeLayer.getBounds(),{padding:[35,35]}); }
  else { routeLayer=null; const bounds=L.latLngBounds(trip.forecasts.map(p=>[p.coordinate.lat,p.coordinate.lon])); if(bounds.isValid())map.fitBounds(bounds,{padding:[45,45]}); }
  forecastTimes=(trip.forecasts[0]?.hours||[]).map(h=>date(h.time)).filter(Boolean); const slider=$('time-slider'); slider.max=Math.max(0,forecastTimes.length-1); const depart=date(trip.departure); let start=forecastTimes.findIndex(t=>t>=depart); if(start<0)start=0; slider.value=start; slider.disabled=!forecastTimes.length; $('end-label').textContent=forecastTimes.at(-1)?.toLocaleDateString([],{month:'short',day:'numeric'})||'—';
  if(coords.length > 1) { $('distance').textContent=`${fmt(trip.distance_km,1)} km`; $('duration').textContent=`${fmt(trip.distance_km/trip.speed_kmh,1)} h`; $('elevation').textContent=`${fmt(trip.ascent_m)} m`; $('summary').classList.remove('hidden'); } else $('summary').classList.add('hidden'); $('alerts').textContent=''; renderAt(start);
}
async function exploreAlps() {
  $('explore').disabled=true; $('status').className='status loading'; $('status').textContent='Fetching forecasts across the Alps…';
  try { const response=await fetch('/api/alps'); const payload=await response.json().catch(()=>({})); if(!response.ok)throw new Error(payload.error||`Server error (${response.status})`); render(payload); $('status').className='status'; $('status').textContent=`Showing ${payload.forecasts.length} interesting locations.`; } catch(error) { $('status').className='status error'; $('status').textContent=`Could not load forecast: ${error.message}`; } finally {$('explore').disabled=false;}
}
async function submit(event) {
  event.preventDefault(); const file=$('gpx').files[0]; if(!file)return; $('submit').disabled=true; $('status').className='status loading'; $('status').textContent='Reading route and fetching forecasts…';
  const body=new FormData(); body.append('gpx',file); body.append('departure_datetime',new Date($('departure').value).toISOString()); body.append('speed_kmh',$('speed').value);
  try { const response=await fetch('/api/trips',{method:'POST',body}); const payload=await response.json().catch(()=>({})); if(!response.ok)throw new Error(payload.error||`Server error (${response.status})`); render(payload); $('status').className='status'; $('status').textContent='Forecast loaded.'; } catch(error) { $('status').className='status error'; $('status').textContent=`Could not load forecast: ${error.message}`; } finally {$('submit').disabled=false;}
}
$('departure').value=localInputDate(new Date(Date.now()+3600000)); $('gpx').addEventListener('change',e=>{if(e.target.files[0])$('file-name').textContent=e.target.files[0].name}); $('dropzone').addEventListener('dragover',e=>e.preventDefault()); $('dropzone').addEventListener('drop',e=>{e.preventDefault();if(e.dataTransfer.files[0]){$('gpx').files=e.dataTransfer.files;$('file-name').textContent=e.dataTransfer.files[0].name}}); $('trip-form').addEventListener('submit',submit); $('explore').addEventListener('click',exploreAlps); $('time-slider').addEventListener('input',e=>renderAt(Number(e.target.value))); $('clear').addEventListener('click',()=>{trip=null;if(routeLayer)routeLayer.remove();markerLayer.clearLayers();$('summary').classList.add('hidden');$('empty-state').classList.remove('hidden');$('time-slider').disabled=true;$('status').textContent=''});
