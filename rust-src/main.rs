use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{ConnectInfo, Request, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::any,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use qrcode::{Color, QrCode};
use rand::{Rng as _, distr::Alphanumeric};
use reqwest::Client;
use rquickjs::{AsyncContext, AsyncRuntime, Function, Promise, function::Async};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tower_http::services::{ServeDir, ServeFile};
use url::Url;
use uuid::Uuid;

mod native;

const COMPAT_SOURCE: &str = include_str!("../rust-assets/compat.js");
const MAX_BODY_BYTES: usize = 100 * 1024 * 1024;

#[derive(Clone)]
struct AppState {
    js: Arc<JsEngine>,
    client: Client,
    device: Arc<DeviceIdentity>,
    cache: Arc<Mutex<HashMap<String, (Instant, ModuleResponse)>>>,
}

struct JsEngine {
    _runtime: AsyncRuntime,
    context: AsyncContext,
}

#[derive(Debug)]
struct DeviceIdentity {
    platform: String,
    guid: String,
    mid: String,
    dev: String,
    mac: String,
    webgl: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct ModuleResponse {
    #[serde(default = "default_status")]
    status: u16,
    #[serde(default)]
    body: Value,
    #[serde(default)]
    cookie: Vec<String>,
    #[serde(default)]
    headers: HashMap<String, Value>,
}

#[derive(Debug, Serialize)]
struct WireBuffer<'a> {
    __kugou_buffer__: &'a str,
}

fn default_status() -> u16 {
    500
}

impl JsEngine {
    async fn new(client: Client, environment: HashMap<String, String>) -> Result<Self, String> {
        let runtime = AsyncRuntime::new()
            .map_err(|error| format!("failed to create JavaScript runtime: {error:?}"))?;
        let context = AsyncContext::full(&runtime)
            .await
            .map_err(|error| format!("failed to create JavaScript context: {error:?}"))?;

        context
            .async_with(async |ctx| {
                let client = client.clone();
                let bridge = Function::new(
                    ctx.clone(),
                    Async(move |input: String| {
                        let client = client.clone();
                        async move { Ok::<_, rquickjs::Error>(raw_http(&client, &input).await) }
                    }),
                )
                .map_err(|error| error.to_string())?;
                ctx.globals()
                    .set("__rust_http", bridge)
                    .map_err(|error| error.to_string())?;
                let qrcode = Function::new(ctx.clone(), |text: String| {
                    qrcode_data_url(&text).map_err(|message| {
                        rquickjs::Error::new_from_js_message("String", "QR code", message)
                    })
                })
                .map_err(|error| error.to_string())?;
                ctx.globals()
                    .set("__rust_qrcode", qrcode)
                    .map_err(|error| error.to_string())?;
                ctx.eval::<(), _>(COMPAT_SOURCE)
                    .map_err(|error| format_js_error(&ctx, error))?;

                let set_env: Function = ctx
                    .globals()
                    .get("__kugou_set_env")
                    .map_err(|error| error.to_string())?;
                let env_json =
                    serde_json::to_string(&environment).map_err(|error| error.to_string())?;
                set_env
                    .call::<_, ()>((env_json,))
                    .map_err(|error| format_js_error(&ctx, error))?;
                Ok::<_, String>(())
            })
            .await?;

        Ok(Self {
            _runtime: runtime,
            context,
        })
    }

    async fn invoke(
        &self,
        module: &str,
        params: Value,
        ip: IpAddr,
    ) -> Result<ModuleResponse, String> {
        let module = module.to_owned();
        let params = serde_json::to_string(&params).map_err(|error| error.to_string())?;
        let ip = ip.to_string();
        let result = self
            .context
            .async_with(async move |ctx| {
                let invoke: Function = ctx
                    .globals()
                    .get("__kugou_invoke")
                    .map_err(|error| error.to_string())?;
                let promise: Promise = invoke
                    .call((module, params, ip))
                    .map_err(|error| format_js_error(&ctx, error))?;
                promise
                    .into_future::<String>()
                    .await
                    .map_err(|error| format_js_error(&ctx, error))
            })
            .await?;

        serde_json::from_str(&result).map_err(|error| format!("invalid module response: {error}"))
    }
}

fn format_js_error(ctx: &rquickjs::Ctx<'_>, error: rquickjs::Error) -> String {
    if error.is_exception() {
        let caught = ctx.catch();
        if let Some(exception) = caught.as_exception() {
            let message = exception.message().unwrap_or_else(|| error.to_string());
            return exception
                .stack()
                .map_or(message.clone(), |stack| format!("{message}\n{stack}"));
        }
    }
    error.to_string()
}

fn qrcode_data_url(text: &str) -> Result<String, String> {
    let code = QrCode::new(text.as_bytes()).map_err(|error| error.to_string())?;
    let module_count = code.width();
    let quiet_zone = 4usize;
    let scale = 4usize;
    let width = (module_count + quiet_zone * 2) * scale;
    let mut pixels = vec![255u8; width * width];
    for y in 0..module_count {
        for x in 0..module_count {
            if code[(x, y)] != Color::Dark {
                continue;
            }
            for offset_y in 0..scale {
                for offset_x in 0..scale {
                    let target_x = (x + quiet_zone) * scale + offset_x;
                    let target_y = (y + quiet_zone) * scale + offset_y;
                    pixels[target_y * width + target_x] = 0;
                }
            }
        }
    }

    let mut png_bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut png_bytes, width as u32, width as u32);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|error| error.to_string())?;
        writer
            .write_image_data(&pixels)
            .map_err(|error| error.to_string())?;
    }
    Ok(format!(
        "data:image/png;base64,{}",
        BASE64.encode(png_bytes)
    ))
}

async fn raw_http(client: &Client, input: &str) -> String {
    match raw_http_inner(client, input).await {
        Ok(response) => serde_json::to_string(&response).unwrap_or_else(wire_json_error),
        Err(error) => serde_json::to_string(&json!({
            "__error__": true,
            "message": error,
        }))
        .unwrap_or_else(wire_json_error),
    }
}

async fn raw_http_inner(client: &Client, input: &str) -> Result<Value, String> {
    let options: Value = serde_json::from_str(input).map_err(|error| error.to_string())?;
    let object = options
        .as_object()
        .ok_or_else(|| "request options must be an object".to_owned())?;
    let raw_url = string_field(object, "url").unwrap_or_default();
    let base_url = string_field(object, "baseURL");
    let mut url = if Url::parse(&raw_url).is_ok() {
        Url::parse(&raw_url).map_err(|error| error.to_string())?
    } else {
        let base = base_url.unwrap_or_else(|| "https://gateway.kugou.com".to_owned());
        Url::parse(&base)
            .map_err(|error| error.to_string())?
            .join(&raw_url)
            .map_err(|error| error.to_string())?
    };

    if let Some(params) = object.get("params").and_then(Value::as_object) {
        append_query(&mut url, params);
    }

    let method = string_field(object, "method")
        .unwrap_or_else(|| "GET".to_owned())
        .to_uppercase();
    let method =
        reqwest::Method::from_bytes(method.as_bytes()).map_err(|error| error.to_string())?;
    let mut request = client.request(method, url);

    if let Some(headers) = object.get("headers").and_then(Value::as_object) {
        for (name, value) in headers {
            if value.is_null() {
                continue;
            }
            request = request.header(name, js_string(value));
        }
    }

    if let Some(data) = object.get("data").filter(|value| !value.is_null()) {
        if let Some(encoded) = data.get("__kugou_buffer__").and_then(Value::as_str) {
            request = request.body(BASE64.decode(encoded).map_err(|error| error.to_string())?);
        } else if data.is_object() || data.is_array() {
            request = request.json(data);
        } else if let Some(text) = data.as_str() {
            request = request.body(text.to_owned());
        } else {
            request = request.body(js_string(data));
        }
    }

    let response = request.send().await.map_err(|error| error.to_string())?;
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    let array_buffer = string_field(object, "responseType").as_deref() == Some("arraybuffer");
    let data = if array_buffer {
        json!(WireBuffer {
            __kugou_buffer__: &BASE64.encode(&bytes),
        })
    } else {
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
    };

    let mut response_headers = Map::new();
    for name in headers.keys() {
        let values: Vec<String> = headers
            .get_all(name)
            .iter()
            .filter_map(|value| value.to_str().ok().map(str::to_owned))
            .collect();
        response_headers.insert(
            name.as_str().to_owned(),
            if name == header::SET_COOKIE {
                json!(values)
            } else {
                Value::String(values.join(", "))
            },
        );
    }

    let response = json!({
        "data": data,
        "headers": response_headers,
        "status": status.as_u16(),
    });
    if status.is_success() {
        Ok(response)
    } else {
        Ok(json!({
            "__error__": true,
            "message": format!("upstream returned HTTP {}", status.as_u16()),
            "response": response,
        }))
    }
}

fn wire_json_error(error: serde_json::Error) -> String {
    format!(
        r#"{{"__error__":true,"message":{}}}"#,
        json!(error.to_string())
    )
}

fn string_field(object: &Map<String, Value>, name: &str) -> Option<String> {
    object
        .get(name)
        .filter(|value| !value.is_null())
        .map(js_string)
}

fn js_string(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn append_query(url: &mut Url, params: &Map<String, Value>) {
    let mut query = url.query_pairs_mut();
    for (name, value) in params {
        match value {
            Value::Array(values) => {
                for value in values {
                    query.append_pair(&format!("{name}[]"), &js_string(value));
                }
            }
            Value::Null => {}
            _ => {
                query.append_pair(name, &js_string(value));
            }
        }
    }
}

async fn api_handler(
    State(state): State<AppState>,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    request: Request,
) -> Response {
    let module = route_to_module(request.uri().path());
    let is_https = request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("https"));
    let origin = request
        .headers()
        .get(header::ORIGIN)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);

    if request.method() == Method::OPTIONS {
        return cors_response(StatusCode::NO_CONTENT.into_response(), origin.as_deref());
    }

    let (parts, body) = request.into_parts();
    let client_ip = forwarded_ip(&parts.headers).unwrap_or_else(|| address.ip().to_string());
    let cache_key = format!(
        "{}{}",
        parts
            .headers
            .get(header::HOST)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default(),
        parts.uri
    );
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let body_limit = if content_type.starts_with("application/octet-stream") {
        MAX_BODY_BYTES
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        5 * 1024 * 1024
    } else {
        16 * 1024 * 1024
    };
    let bytes = match axum::body::to_bytes(body, body_limit).await {
        Ok(bytes) => bytes,
        Err(error) => {
            return cors_response(
                (
                    StatusCode::PAYLOAD_TOO_LARGE,
                    json!({ "status": 0, "msg": error.to_string() }).to_string(),
                )
                    .into_response(),
                origin.as_deref(),
            );
        }
    };

    let (params, device_cookies) = match build_params(&parts, bytes, &state.device) {
        Ok(value) => value,
        Err(message) => {
            return cors_response(
                (
                    StatusCode::BAD_REQUEST,
                    json!({ "status": 0, "msg": message }).to_string(),
                )
                    .into_response(),
                origin.as_deref(),
            );
        }
    };

    let no_cookie = params.get("noCookie").is_some_and(truthy);
    let cached = state.cache.lock().ok().and_then(|mut cache| {
        let (stored_at, response) = cache.get(&cache_key)?;
        if stored_at.elapsed() < Duration::from_secs(120) {
            Some(response.clone())
        } else {
            cache.remove(&cache_key);
            None
        }
    });
    let module_response = if let Some(response) = cached {
        response
    } else {
        let result = if native::supports(&module) {
            native::invoke(
                &module,
                &state.client,
                &params,
                client_ip.parse().unwrap_or(address.ip()),
            )
            .await
        } else {
            state
                .js
                .invoke(&module, params, client_ip.parse().unwrap_or(address.ip()))
                .await
        };
        let response = match result {
            Ok(response) => response,
            Err(message) => ModuleResponse {
                status: 404,
                body: json!({ "code": 404, "data": null, "msg": "Not Found", "error": message }),
                cookie: Vec::new(),
                headers: HashMap::new(),
            },
        };
        if response.status == 200
            && let Ok(mut cache) = state.cache.lock()
        {
            cache.insert(cache_key, (Instant::now(), response.clone()));
        }
        response
    };

    let status =
        StatusCode::from_u16(module_response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut response = response_from_body(status, module_response.body);
    for (name, value) in module_response.headers {
        let Ok(name) = HeaderName::try_from(name) else {
            continue;
        };
        let text = js_string(&value);
        if let Ok(value) = HeaderValue::try_from(text) {
            response.headers_mut().insert(name, value);
        }
    }

    let cookie_suffix = if is_https {
        "; PATH=/; SameSite=None; Secure"
    } else {
        "; PATH=/"
    };
    for cookie in device_cookies {
        append_header(
            response.headers_mut(),
            header::SET_COOKIE,
            format!("{cookie}{cookie_suffix}"),
        );
    }
    if !no_cookie {
        for cookie in module_response.cookie {
            append_header(
                response.headers_mut(),
                header::SET_COOKIE,
                format!("{cookie}{cookie_suffix}"),
            );
        }
    }

    println!(
        "[{}] {}",
        if status.is_success() { "OK" } else { "ERR" },
        parts.uri
    );
    cors_response(response, origin.as_deref())
}

fn response_from_body(status: StatusCode, body: Value) -> Response {
    if let Some(encoded) = body.get("__kugou_buffer__").and_then(Value::as_str) {
        let bytes = BASE64.decode(encoded).unwrap_or_default();
        return (status, bytes).into_response();
    }
    let text = match body {
        Value::String(text) => text,
        value => serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_owned()),
    };
    let mut response = (status, Body::from(text)).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    response
}

fn cors_response(mut response: Response, origin: Option<&str>) -> Response {
    let allow_origin = std::env::var("CORS_ALLOW_ORIGIN")
        .ok()
        .or_else(|| origin.map(str::to_owned))
        .unwrap_or_else(|| "*".to_owned());
    append_header(
        response.headers_mut(),
        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
        "true".to_owned(),
    );
    append_header(
        response.headers_mut(),
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        allow_origin,
    );
    append_header(
        response.headers_mut(),
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        "Authorization,X-Requested-With,Content-Type,Cache-Control".to_owned(),
    );
    append_header(
        response.headers_mut(),
        header::ACCESS_CONTROL_ALLOW_METHODS,
        "PUT,POST,GET,DELETE,OPTIONS".to_owned(),
    );
    response
}

fn append_header(headers: &mut HeaderMap, name: HeaderName, value: String) {
    if let Ok(value) = HeaderValue::try_from(value) {
        headers.append(name, value);
    }
}

fn forwarded_ip(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

fn build_params(
    parts: &axum::http::request::Parts,
    bytes: Bytes,
    device: &DeviceIdentity,
) -> Result<(Value, Vec<String>), String> {
    let mut query = parse_pairs(parts.uri.query().unwrap_or_default().as_bytes());
    let mut body = parse_body(&parts.headers, bytes)?;
    parse_cookie_field(&mut query);
    parse_cookie_field(&mut body);

    let mut cookies = parse_cookie_header(parts.headers.get(header::COOKIE));
    let mut device_cookies = Vec::new();
    for (name, value) in [
        ("KUGOU_API_PLATFORM", device.platform.as_str()),
        ("KUGOU_API_MID", device.mid.as_str()),
        ("KUGOU_API_GUID", device.guid.as_str()),
        ("KUGOU_API_DEV", device.dev.as_str()),
        ("KUGOU_API_MAC", device.mac.as_str()),
    ] {
        if !cookies.contains_key(name) {
            cookies.insert(name.to_owned(), Value::String(value.to_owned()));
            device_cookies.push(format!("{name}={value}"));
        }
    }
    if !cookies.contains_key("KUGOU_API_WEBGL") {
        let webgl = device
            .webgl
            .clone()
            .unwrap_or_else(|| rand::random::<u64>().to_string());
        cookies.insert("KUGOU_API_WEBGL".to_owned(), Value::String(webgl.clone()));
        device_cookies.push(format!("KUGOU_API_WEBGL={webgl}"));
    }

    if let Some(value) = query.remove("cookie") {
        merge_cookie_value(&mut cookies, value);
    }

    let mut params = query;
    params.insert("cookie".to_owned(), Value::Object(cookies));
    params.extend(body);

    if let Some(auth) = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        && let Some(cookie) = params.get_mut("cookie").and_then(Value::as_object_mut)
    {
        cookie.extend(parse_loose_cookie(auth));
    }

    Ok((Value::Object(params), device_cookies))
}

fn parse_body(headers: &HeaderMap, bytes: Bytes) -> Result<Map<String, Value>, String> {
    if bytes.is_empty() {
        return Ok(Map::new());
    }
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    if content_type.starts_with("application/octet-stream") {
        let encoded = BASE64.encode(bytes);
        return Ok(Map::from_iter([(
            "data".to_owned(),
            json!(WireBuffer {
                __kugou_buffer__: &encoded,
            }),
        )]));
    }
    if content_type.starts_with("application/x-www-form-urlencoded") {
        return Ok(parse_pairs(&bytes));
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    value
        .as_object()
        .cloned()
        .ok_or_else(|| "JSON request body must be an object".to_owned())
}

fn parse_pairs(input: &[u8]) -> Map<String, Value> {
    let mut result = Map::new();
    for (name, value) in url::form_urlencoded::parse(input) {
        let name = name.into_owned();
        let value = Value::String(value.into_owned());
        match result.get_mut(&name) {
            Some(Value::Array(items)) => items.push(value),
            Some(previous) => {
                let old = previous.take();
                *previous = Value::Array(vec![old, value]);
            }
            None => {
                result.insert(name, value);
            }
        }
    }
    result
}

fn parse_cookie_header(value: Option<&HeaderValue>) -> Map<String, Value> {
    let Some(value) = value.and_then(|value| value.to_str().ok()) else {
        return Map::new();
    };
    value
        .split(';')
        .filter_map(|pair| {
            let (name, value) = pair.trim().split_once('=')?;
            if name.is_empty() || value.is_empty() {
                return None;
            }
            Some((name.to_owned(), Value::String(percent_decode(value))))
        })
        .collect()
}

fn parse_loose_cookie(value: &str) -> Map<String, Value> {
    value
        .split(';')
        .map(|pair| {
            let mut parts = pair.split('=');
            (
                parts.next().unwrap_or_default().to_owned(),
                Value::String(parts.next().unwrap_or_default().to_owned()),
            )
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    url::form_urlencoded::parse(format!("v={value}").as_bytes())
        .next()
        .map(|(_, value)| value.into_owned())
        .unwrap_or_default()
}

fn parse_cookie_field(object: &mut Map<String, Value>) {
    let Some(Value::String(value)) = object.get("cookie") else {
        return;
    };
    let parsed = Value::Object(parse_loose_cookie(value));
    object.insert("cookie".to_owned(), parsed);
}

fn merge_cookie_value(target: &mut Map<String, Value>, value: Value) {
    if let Value::Object(value) = value {
        target.extend(value);
    }
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|value| value != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

fn route_to_module(path: &str) -> String {
    path.trim_matches('/').replace('/', "_")
}

fn module_to_route(module: &str) -> String {
    format!("/{}", module.replace('_', "/"))
}

fn device_identity() -> DeviceIdentity {
    let raw_guid = std::env::var("KUGOU_API_GUID").unwrap_or_else(|_| Uuid::new_v4().to_string());
    let guid = if Uuid::parse_str(&raw_guid).is_ok_and(|value| value.get_version_num() == 4) {
        format!("{:x}", md5::compute(raw_guid))
    } else {
        raw_guid
    };
    let mid_digest = md5::compute(&guid);
    let mid = u128::from_be_bytes(mid_digest.0).to_string();
    let dev = std::env::var("KUGOU_API_DEV")
        .unwrap_or_else(|_| {
            rand::rng()
                .sample_iter(Alphanumeric)
                .take(10)
                .map(char::from)
                .collect()
        })
        .to_uppercase();
    DeviceIdentity {
        platform: std::env::var("platform").unwrap_or_else(|_| "undefined".to_owned()),
        guid,
        mid,
        dev,
        mac: std::env::var("KUGOU_API_MAC")
            .unwrap_or_else(|_| "02:00:00:00:00:00".to_owned())
            .to_uppercase(),
        webgl: std::env::var("KUGOU_API_WEBGL").ok(),
    }
}

fn environment() -> HashMap<String, String> {
    std::env::vars().collect()
}

fn http_client() -> Result<Client, String> {
    let mut builder = Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent("Android15-1070-11083-46-0-DiscoveryDRADProtocol-wifi");
    if let Ok(proxy) = std::env::var("KUGOU_API_PROXY")
        && !proxy.trim().is_empty()
    {
        builder =
            builder.proxy(reqwest::Proxy::all(proxy.trim()).map_err(|error| error.to_string())?);
    }
    builder
        .build()
        .map_err(|error| format!("failed to build HTTP client: {error:?}"))
}

fn load_dotenv() {
    let path = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".env");
    if path.exists() {
        let _ = dotenvy::from_path(path);
    }
}

fn apply_cli_overrides() {
    for argument in std::env::args().skip(1) {
        if let Some(value) = argument.strip_prefix("--proxy=") {
            // SAFETY: CLI overrides run before Tokio or the JS runtime starts any threads.
            unsafe { std::env::set_var("KUGOU_API_PROXY", value) };
        } else if let Some(value) = argument.strip_prefix("--platform=") {
            // SAFETY: CLI overrides run before Tokio or the JS runtime starts any threads.
            unsafe { std::env::set_var("platform", value) };
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    load_dotenv();
    apply_cli_overrides();

    let client = http_client()?;
    let js = Arc::new(
        JsEngine::new(client.clone(), environment())
            .await
            .map_err(|error| format!("failed to initialize compatibility runtime: {error}"))?,
    );
    let mut modules = js
        .context
        .async_with(async |ctx| {
            ctx.globals()
                .get::<_, Vec<String>>("__kugou_modules")
                .map_err(|error| error.to_string())
        })
        .await
        .map_err(|error| format!("failed to read embedded API modules: {error}"))?;
    modules.extend(native::modules()?);
    modules.sort();
    modules.dedup();
    let state = AppState {
        js,
        client,
        device: Arc::new(device_identity()),
        cache: Arc::new(Mutex::new(HashMap::new())),
    };

    let mut app = Router::new();
    for module in &modules {
        app = app.route(&module_to_route(module), any(api_handler));
    }
    let health_token = std::env::var("KUGOU_API_HEALTH_TOKEN").ok();
    if let Some(token) = health_token {
        let path = std::env::var("KUGOU_API_HEALTH_PATH")
            .unwrap_or_else(|_| "/__hydrogen/health".to_owned());
        app = app.route(
            &path,
            any(move || {
                let token = token.clone();
                async move { axum::Json(json!({ "service": token })) }
            }),
        );
    }

    let public = ServeDir::new("public").not_found_service(ServeFile::new("public/index.html"));
    let app = app
        .nest_service(
            "/docs",
            ServeDir::new("docs").append_index_html_on_directories(true),
        )
        .fallback_service(public)
        .with_state(state);

    let host = std::env::var("HOST").unwrap_or_default();
    let host = if host.is_empty() { "0.0.0.0" } else { &host };
    let port = std::env::var("PORT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "3000".to_owned())
        .parse::<u16>()
        .map_err(|error| format!("invalid PORT: {error}"))?;
    let address: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|error: std::net::AddrParseError| format!("invalid listen address: {error}"))?;
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .map_err(|error| format!("failed to bind {address}: {error}"))?;
    println!(
        "server running @ http://{}:{}",
        if host == "0.0.0.0" { "localhost" } else { host },
        port
    );
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .map_err(|error| format!("server failed: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_mapping_matches_node_file_convention() {
        assert_eq!(module_to_route("user_cloud_url"), "/user/cloud/url");
        assert_eq!(route_to_module("/user/cloud/url"), "user_cloud_url");
    }

    #[test]
    fn loose_cookie_keeps_node_compatibility() {
        let cookie = parse_loose_cookie("token=abc;userid=123");
        assert_eq!(cookie["token"], "abc");
        assert_eq!(cookie["userid"], "123");
    }

    #[test]
    fn repeated_query_values_become_an_array() {
        let params = parse_pairs(b"id=1&id=2");
        assert_eq!(params["id"], json!(["1", "2"]));
    }
}
