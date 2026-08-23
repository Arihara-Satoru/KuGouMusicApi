use super::*;

const NATIVE_MODULES: &str = include_str!("../rust-native.json");

pub fn modules() -> Result<Vec<String>, String> {
    serde_json::from_str(NATIVE_MODULES).map_err(|error| error.to_string())
}

pub fn supports(module: &str) -> bool {
    matches!(
        module,
        "album_shop"
            | "artist_detail"
            | "everyday_recommend"
            | "favorite_count"
            | "longaudio_daily_recommend"
            | "longaudio_rank_recommend"
            | "longaudio_vip_recommend"
            | "longaudio_week_recommend"
            | "login_qr_create"
            | "login_wx_check"
            | "rank_top"
            | "scene_lists"
            | "scene_module"
            | "scene_module_info"
            | "sheet_detail"
            | "sheet_tags"
            | "song_ranking"
            | "user_vip_detail"
            | "youth_channel_all"
            | "youth_dynamic"
            | "youth_dynamic_recent"
            | "youth_month_vip_record"
    )
}

pub async fn invoke(
    module: &str,
    client: &Client,
    params: &Value,
    ip: IpAddr,
) -> Result<ModuleResponse, String> {
    match module {
        "album_shop" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/zhuanjidata/v3/album_shop_v2/get_classify_data"),
            )
            .await
        }
        "artist_detail" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/kmr/v3/author")
                    .data(json!({ "author_id": value(params, "id") }))
                    .header("x-router", "openapi.kugou.com")
                    .header("kg-tid", "36"),
            )
            .await
        }
        "everyday_recommend" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/everyday_song_recommend")
                    .params(json!({ "platform": value_or(params, "platform", json!("ios")) }))
                    .header("x-router", "everydayrec.service.kugou.com"),
            )
            .await
        }
        "favorite_count" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/count/v1/audio/mget_collect")
                    .params(json!({ "mixsongids": value(params, "mixsongids") })),
            )
            .await
        }
        "longaudio_daily_recommend" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/longaudio/v1/home_new/daily_recommend").params(json!({
                    "module_id": 1,
                    "size": value_or(params, "pagesize", json!(30)),
                    "page": value_or(params, "page", json!(1)),
                })),
            )
            .await
        }
        "longaudio_rank_recommend" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/longaudio/v1/home_new/rank_card_recommend")
                    .params(json!({ "platform": "ios" })),
            )
            .await
        }
        "longaudio_vip_recommend" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/longaudio/v1/home_new/vip_select_recommend")
                    .params(json!({ "position": "2", "clientver": 12329 }))
                    .data(json!({ "album_playlist": [] })),
            )
            .await
        }
        "longaudio_week_recommend" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/longaudio/v1/home_new/week_new_albums_recommend")
                    .params(json!({ "clientver": 12329 }))
                    .data(json!({ "album_playlist": [] })),
            )
            .await
        }
        "login_qr_create" => login_qr_create(params),
        "login_wx_check" => login_wx_check(client, params).await,
        "rank_top" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/mobileservice/api/v5/rank/rec_rank_list"),
            )
            .await
        }
        "scene_lists" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/scene/v1/scene/list"),
            )
            .await
        }
        "scene_module" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/scene/v1/scene/module")
                    .params(json!({ "scene_id": value(params, "id") })),
            )
            .await
        }
        "scene_module_info" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/scene/v1/scene/module_info").params(json!({
                    "scene_id": value(params, "id"),
                    "module_id": value(params, "module_id"),
                })),
            )
            .await
        }
        "sheet_detail" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/opern/v1/detail/info")
                    .params(json!({ "opern_id": value(params, "id") })),
            )
            .await
        }
        "sheet_tags" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/opern/v1/home/get_tags"),
            )
            .await
        }
        "song_ranking" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/grow/v1/song_ranking/play_page/ranking_info")
                    .params(json!({ "album_audio_id": value(params, "album_audio_id") })),
            )
            .await
        }
        "user_vip_detail" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/get_union_vip")
                    .base_url("https://kugouvip.kugou.com")
                    .params(json!({ "busi_type": "concept" })),
            )
            .await
        }
        "youth_channel_all" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/v2/channel/channel_all_list").params(json!({
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "type": 1,
                })),
            )
            .await
        }
        "youth_dynamic" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/v3/user/get_dynamic"),
            )
            .await
        }
        "youth_dynamic_recent" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/v3/user/recent_dynamic"),
            )
            .await
        }
        "youth_month_vip_record" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/v1/activity/get_month_vip_record")
                    .params(json!({ "latest_limit": 100 })),
            )
            .await
        }
        _ => Err(format!("unknown native module: {module}")),
    }
}

struct NativeRequest {
    url: &'static str,
    method: Method,
    params: Map<String, Value>,
    data: Option<Value>,
    headers: HashMap<&'static str, &'static str>,
    base_url: &'static str,
}

impl NativeRequest {
    fn get(url: &'static str) -> Self {
        Self::new(url, Method::GET)
    }

    fn post(url: &'static str) -> Self {
        Self::new(url, Method::POST)
    }

    fn new(url: &'static str, method: Method) -> Self {
        Self {
            url,
            method,
            params: Map::new(),
            data: None,
            headers: HashMap::new(),
            base_url: "https://gateway.kugou.com",
        }
    }

    fn params(mut self, params: Value) -> Self {
        self.params = params.as_object().cloned().unwrap_or_default();
        self
    }

    fn data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }

    fn header(mut self, name: &'static str, value: &'static str) -> Self {
        self.headers.insert(name, value);
        self
    }

    fn base_url(mut self, base_url: &'static str) -> Self {
        self.base_url = base_url;
        self
    }
}

fn value(params: &Value, name: &str) -> Value {
    params.get(name).cloned().unwrap_or(Value::Null)
}

fn value_or(params: &Value, name: &str, default: Value) -> Value {
    params
        .get(name)
        .filter(|value| truthy(value))
        .cloned()
        .unwrap_or(default)
}

async fn android_request(
    client: &Client,
    input: &Value,
    ip: IpAddr,
    options: NativeRequest,
) -> Result<ModuleResponse, String> {
    Ok(
        match android_request_inner(client, input, ip, options).await {
            Ok(response) => response,
            Err(message) => upstream_error(message),
        },
    )
}

async fn android_request_inner(
    client: &Client,
    input: &Value,
    ip: IpAddr,
    options: NativeRequest,
) -> Result<ModuleResponse, String> {
    let cookie = input
        .get("cookie")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let dfid = cookie
        .get("dfid")
        .filter(|value| truthy(value))
        .map(js_string)
        .unwrap_or_else(|| "-".to_owned());
    let mid = cookie
        .get("KUGOU_API_MID")
        .map(js_string)
        .unwrap_or_else(|| "undefined".to_owned());
    let token = cookie
        .get("token")
        .filter(|value| truthy(value))
        .map(js_string)
        .unwrap_or_default();
    let userid = cookie
        .get("userid")
        .filter(|value| truthy(value))
        .cloned()
        .unwrap_or_else(|| json!(0));
    let clienttime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let is_lite = std::env::var("platform").as_deref() == Ok("lite");

    let mut query = Map::from_iter([
        ("dfid".to_owned(), Value::String(dfid.clone())),
        ("mid".to_owned(), Value::String(mid.clone())),
        ("uuid".to_owned(), Value::String("-".to_owned())),
        ("appid".to_owned(), json!(if is_lite { 3116 } else { 1005 })),
        (
            "clientver".to_owned(),
            json!(if is_lite { 11440 } else { 20489 }),
        ),
        ("clienttime".to_owned(), json!(clienttime)),
    ]);
    if !token.is_empty() {
        query.insert("token".to_owned(), Value::String(token));
    }
    if truthy(&userid) {
        query.insert("userid".to_owned(), userid);
    }
    query.extend(options.params);

    let data = options.data.as_ref().map_or_else(String::new, js_string);
    let signature = signature_android(&query, &data, is_lite);
    query.insert("signature".to_owned(), Value::String(signature));

    let mut url = Url::parse(options.base_url)
        .map_err(|error| error.to_string())?
        .join(options.url)
        .map_err(|error| error.to_string())?;
    append_query(&mut url, &query);
    let mut request = client.request(options.method, url);
    for (name, value) in options.headers {
        request = request.header(name, value);
    }
    request = request
        .header("dfid", &dfid)
        .header("clienttime", clienttime)
        .header("mid", &mid)
        .header("kg-rc", "1")
        .header("kg-thash", "5d816a0")
        .header("kg-rec", "1")
        .header("kg-rf", "B9EDA08A64250DEFFBCADDEE00F8F25F")
        .header("X-Real-IP", ip.to_string())
        .header("X-Forwarded-For", ip.to_string());
    if let Some(data) = options.data {
        request = if data.is_object() || data.is_array() {
            request.json(&data)
        } else {
            request.body(js_string(&data))
        };
    }

    let response = request.send().await.map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Ok(upstream_error(format!(
            "upstream returned HTTP {}",
            response.status().as_u16()
        )));
    }
    let response_headers = response.headers().clone();
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    let mut body = serde_json::from_slice(&bytes)
        .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()));
    let failed = body.get("status").and_then(Value::as_i64) == Some(0)
        || body
            .get("error_code")
            .is_some_and(|value| truthy(value) && value.as_i64() != Some(0));
    let cookies = response_headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok().map(clean_set_cookie))
        .collect();
    let mut headers = HashMap::new();
    if let Some(ssa_code) = response_headers
        .get("ssa-code")
        .and_then(|value| value.to_str().ok())
    {
        headers.insert("ssa-code".to_owned(), json!(ssa_code));
        if let Some(object) = body.as_object_mut() {
            // ponytail: authenticated modules stay on JS until the SSA fingerprint generator is ported.
            object.insert("ssaCode".to_owned(), json!(ssa_code));
        }
    }
    Ok(ModuleResponse {
        status: if failed { 502 } else { 200 },
        body,
        cookie: cookies,
        headers,
    })
}

fn signature_android(params: &Map<String, Value>, data: &str, is_lite: bool) -> String {
    let salt = if is_lite {
        "LnT6xpN3khm36zse0QzvmgTZ3waWdRSA"
    } else {
        "OIlwieks28dk2k092lksi2UIkp"
    };
    let mut entries: Vec<_> = params.iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
    let params = entries
        .into_iter()
        .map(|(key, value)| format!("{key}={}", js_string(value)))
        .collect::<String>();
    format!("{:x}", md5::compute(format!("{salt}{params}{data}{salt}")))
}

fn clean_set_cookie(value: &str) -> String {
    value
        .split(';')
        .map(str::trim)
        .filter(|part| {
            let lower = part.to_ascii_lowercase();
            !lower.starts_with("domain=")
                && !lower.starts_with("path=")
                && !lower.starts_with("expires=")
                && lower != "httponly"
        })
        .collect::<Vec<_>>()
        .join(";")
}

fn upstream_error(message: String) -> ModuleResponse {
    ModuleResponse {
        status: 502,
        body: json!({ "status": 0, "msg": message }),
        cookie: Vec::new(),
        headers: HashMap::new(),
    }
}

fn login_qr_create(params: &Value) -> Result<ModuleResponse, String> {
    let key = params
        .get("key")
        .map(js_string)
        .unwrap_or_else(|| "undefined".to_owned());
    let url = format!("https://h5.kugou.com/apps/loginQRCode/html/index.html?qrcode={key}");
    let base64 = if params.get("qrimg").is_some_and(truthy) {
        qrcode_data_url(&url)?
    } else {
        String::new()
    };
    Ok(ModuleResponse {
        status: 200,
        body: json!({
            "code": 200,
            "data": {
                "url": url,
                "base64": base64,
            },
        }),
        cookie: Vec::new(),
        headers: HashMap::new(),
    })
}

async fn login_wx_check(client: &Client, params: &Value) -> Result<ModuleResponse, String> {
    let uuid = params.get("uuid").map(js_string).unwrap_or_default();
    let result = async {
        let response = client
            .get("https://long.open.weixin.qq.com/connect/l/qrconnect")
            .query(&[("f", "json"), ("uuid", uuid.as_str())])
            .send()
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let bytes = response.bytes().await.map_err(|error| error.to_string())?;
        Ok::<_, String>(
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned())),
        )
    }
    .await;

    Ok(match result {
        Ok(body) => ModuleResponse {
            status: 200,
            body,
            cookie: Vec::new(),
            headers: HashMap::new(),
        },
        Err(message) => ModuleResponse {
            status: 502,
            body: json!({ "status": 0, "msg": message }),
            cookie: Vec::new(),
            headers: HashMap::new(),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_only_lists_implemented_handlers() {
        let modules = modules().expect("native manifest should be valid JSON");
        assert_eq!(modules.len(), 22);
        assert!(modules.iter().all(|module| supports(module)));
    }

    #[test]
    fn qr_without_image_matches_node_response() {
        let response = login_qr_create(&json!({ "key": "contract-test" })).unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(
            response.body,
            json!({
                "code": 200,
                "data": {
                    "url": "https://h5.kugou.com/apps/loginQRCode/html/index.html?qrcode=contract-test",
                    "base64": "",
                },
            })
        );
    }

    #[test]
    fn android_signature_matches_node_vector() {
        let params = json!({
            "appid": 3116,
            "clienttime": 1700000000,
            "clientver": 11440,
            "dfid": "-",
            "mid": "123",
            "uuid": "-",
        });
        assert_eq!(
            signature_android(params.as_object().unwrap(), "", true),
            "154afc2378c777465a2b712468a7f9f9"
        );
    }
}
