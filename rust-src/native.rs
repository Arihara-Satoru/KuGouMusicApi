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
            | "artist_honour"
            | "everyday_recommend"
            | "everyday_style_recommend"
            | "favorite_count"
            | "ip_dateil"
            | "ip_playlist"
            | "ip_zone_home"
            | "longaudio_daily_recommend"
            | "longaudio_rank_recommend"
            | "longaudio_vip_recommend"
            | "longaudio_week_recommend"
            | "login_qr_create"
            | "login_wx_check"
            | "pc_diantai"
            | "playlist_effect"
            | "playlist_tags"
            | "rank_list"
            | "rank_top"
            | "rank_vol"
            | "scene_lists"
            | "scene_music"
            | "scene_module"
            | "scene_module_info"
            | "sheet_detail"
            | "sheet_collection"
            | "sheet_tags"
            | "singer_list"
            | "song_climax"
            | "song_ranking"
            | "song_ranking_filter"
            | "search_hot"
            | "user_vip_detail"
            | "youth_channel_all"
            | "youth_channel_amway"
            | "youth_channel_detail"
            | "youth_channel_song_detail"
            | "youth_channel_sub"
            | "youth_dynamic"
            | "youth_dynamic_recent"
            | "youth_month_vip_record"
            | "youth_union_vip"
            | "youth_vip"
            | "yueku"
            | "yueku_fm"
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
        "artist_honour" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/query_singer_honour_detail")
                    .base_url("http://h5activity.kugou.com")
                    .params(json!({
                        "singer_id": value(params, "id"),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "page": value_or(params, "page", json!(1)),
                    })),
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
        "everyday_style_recommend" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/everydayrec.service/everyday_style_recommend")
                    .params(json!({ "tagids": value_nullish(params, "tagids", json!("")) }))
                    .data(json!({})),
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
        "ip_dateil" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/openapi/v1/ip").data(json!({
                    "data": csv_objects(params, "id", "ip_id"),
                    "is_publish": 1,
                })),
            )
            .await
        }
        "ip_playlist" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/ocean/v6/pubsongs/list_info_for_ip").params(json!({
                    "ip": value(params, "id"),
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                })),
            )
            .await
        }
        "ip_zone_home" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/zone/home")
                    .params(json!({ "id": value(params, "id"), "share": 0 }))
                    .header("x-router", "yuekucategory.kugou.com"),
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
        "pc_diantai" => {
            let userid = params
                .get("cookie")
                .and_then(Value::as_object)
                .and_then(|cookie| cookie.get("userid"))
                .filter(|value| truthy(value))
                .cloned()
                .unwrap_or_else(|| value_or(params, "userid", json!(0)));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v3/pc_diantai")
                    .base_url("https://adservice.kugou.com")
                    .data(json!({ "isvip": 0, "userid": userid, "vipType": 0 })),
            )
            .await
        }
        "playlist_effect" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/pubsongs/v1/get_sound_effect_list").data(json!({
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                })),
            )
            .await
        }
        "playlist_tags" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/pubsongs/v1/get_tags_by_type").data(json!({
                    "tag_type": "collection",
                    "tag_id": 0,
                    "source": 3,
                })),
            )
            .await
        }
        "rank_list" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/ocean/v6/rank/list").params(json!({
                    "plat": 2,
                    "withsong": value_or(params, "withsong", json!(1)),
                    "parentid": 0,
                })),
            )
            .await
        }
        "rank_top" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/mobileservice/api/v5/rank/rec_rank_list"),
            )
            .await
        }
        "rank_vol" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/ocean/v6/rank/vol").params(json!({
                    "rank_cid": value_or(params, "rank_cid", json!(0)),
                    "rankid": value(params, "rankid"),
                    "ranktype": 1,
                    "type": 0,
                    "plat": 2,
                })),
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
        "scene_music" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/genesisapi/v1/scene_music/rec_music")
                    .params(json!({
                        "scene_id": value(params, "id"),
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                    }))
                    .data(json!({ "exposure": [] })),
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
        "search_hot" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/api/v3/search/hot_tab")
                    .params(json!({ "navid": 1, "plat": 2 }))
                    .header("x-router", "msearch.kugou.com"),
            )
            .await
        }
        "sheet_collection" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/miniyueku/v1/opern_square/get_home_module_config")
                    .params(json!({
                        "srcappid": 2919,
                        "position": value_nullish(params, "position", json!(2)),
                    }))
                    .web(),
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
        "singer_list" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/ocean/v6/singer/list").params(json!({
                    "hotsize": value_nullish(params, "hotsize", json!(200)),
                    "musician": 0,
                    "sextype": value_nullish(params, "sextype", json!(0)),
                    "showtype": 2,
                    "type": value_nullish(params, "type", json!(0)),
                })),
            )
            .await
        }
        "song_climax" => {
            let data = csv_objects(params, "hash", "hash");
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/audio_climax/audio")
                    .base_url("https://expendablekmrcdn.kugou.com")
                    .params(json!({ "data": js_string(&data) })),
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
        "song_ranking_filter" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/grow/v1/song_ranking/unlock/v2/ranking_filter").params(
                    json!({
                        "album_audio_id": value(params, "album_audio_id"),
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                    }),
                ),
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
        "youth_channel_amway" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/api/amway/v2/index").params(json!({
                    "global_collection_id": value(params, "global_collection_id"),
                })),
            )
            .await
        }
        "youth_channel_detail" => {
            let data = csv_objects(params, "global_collection_id", "global_collection_id");
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/api/channel/v1/channel_list_by_id")
                    .data(json!({ "data": data })),
            )
            .await
        }
        "youth_channel_song_detail" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/v2/post/get_song_detail").params(json!({
                    "global_collection_id": value(params, "global_collection_id"),
                    "fileid": value(params, "fileid"),
                })),
            )
            .await
        }
        "youth_channel_sub" => {
            let unsubscribe = number_is_zero(params.get("t"));
            let options = if unsubscribe {
                NativeRequest::delete("/youth/v1/channel_un_subscribe")
            } else {
                NativeRequest::post("/youth/v1/channel_subscribe")
            };
            android_request(
                client,
                params,
                ip,
                options.params(json!({
                    "global_collection_id": value(params, "global_collection_id"),
                    "source": 1,
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
        "youth_union_vip" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/get_union_vip")
                    .base_url("https://kugouvip.kugou.com")
                    .params(json!({
                        "busi_type": "concept",
                        "opt_product_types": "dvip,qvip",
                        "product_type": "svip",
                    })),
            )
            .await
        }
        "youth_vip" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/v1/ad/play_report").data(json!({
                    "ad_id": 12307537187u64,
                    "play_end": now,
                    "play_start": now.saturating_sub(30_000),
                })),
            )
            .await
        }
        "yueku" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/yueku/recommend_v2")
                    .params(json!({
                        "operator": 7,
                        "plat": 0,
                        "type": 11,
                        "area_code": 1,
                        "req_multi": 1,
                    }))
                    .header("x-router", "service.mobile.kugou.com"),
            )
            .await
        }
        "yueku_fm" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/time_fm_info")
                    .params(json!({
                        "operator": 7,
                        "plat": 0,
                        "type": 11,
                        "area_code": 1,
                        "req_multi": 1,
                    }))
                    .header("x-router", "fm.service.kugou.com"),
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
    signature: SignatureKind,
}

#[derive(Clone, Copy)]
enum SignatureKind {
    Android,
    Web,
}

impl NativeRequest {
    fn get(url: &'static str) -> Self {
        Self::new(url, Method::GET)
    }

    fn post(url: &'static str) -> Self {
        Self::new(url, Method::POST)
    }

    fn delete(url: &'static str) -> Self {
        Self::new(url, Method::DELETE)
    }

    fn new(url: &'static str, method: Method) -> Self {
        Self {
            url,
            method,
            params: Map::new(),
            data: None,
            headers: HashMap::new(),
            base_url: "https://gateway.kugou.com",
            signature: SignatureKind::Android,
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

    fn web(mut self) -> Self {
        self.signature = SignatureKind::Web;
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

fn value_nullish(params: &Value, name: &str, default: Value) -> Value {
    params
        .get(name)
        .filter(|value| !value.is_null())
        .cloned()
        .unwrap_or(default)
}

fn csv_objects(params: &Value, name: &str, field: &str) -> Value {
    let values = params
        .get(name)
        .filter(|value| truthy(value))
        .map(js_string)
        .unwrap_or_default();
    Value::Array(
        values
            .split(',')
            .map(|value| Value::Object(Map::from_iter([(field.to_owned(), json!(value))])))
            .collect(),
    )
}

fn number_is_zero(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Null) => true,
        Some(Value::Bool(value)) => !value,
        Some(Value::Number(value)) => value.as_f64() == Some(0.0),
        Some(Value::String(value)) => value.trim().parse::<f64>() == Ok(0.0),
        _ => false,
    }
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
    let signature = match options.signature {
        SignatureKind::Android => signature_android(&query, &data, is_lite),
        SignatureKind::Web => signature_web(&query),
    };
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

fn signature_web(params: &Map<String, Value>) -> String {
    let salt = "NVPh5oo715z5DIWAeQlhMDsWXXQV4hwt";
    let mut entries: Vec<_> = params
        .iter()
        .map(|(key, value)| format!("{key}={}", js_string(value)))
        .collect();
    entries.sort();
    format!(
        "{:x}",
        md5::compute(format!("{salt}{}{salt}", entries.concat()))
    )
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
        assert_eq!(modules.len(), 46);
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

    #[test]
    fn web_signature_matches_node_vector() {
        let params = json!({
            "appid": 1005,
            "clienttime": 1700000000,
            "clientver": 20489,
            "dfid": "-",
            "mid": "123",
            "position": 2,
            "srcappid": 2919,
            "uuid": "-",
        });
        assert_eq!(
            signature_web(params.as_object().unwrap()),
            "13eb297ff8d9214a6354a67e02a0665f"
        );
    }
}
