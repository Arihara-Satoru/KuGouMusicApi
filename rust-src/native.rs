use super::*;

const NATIVE_MODULES: &str = include_str!("../rust-native.json");

pub fn modules() -> Result<Vec<String>, String> {
    Ok(module_manifest()?.clone())
}

pub fn supports(module: &str) -> bool {
    module_manifest().is_ok_and(|modules| modules.iter().any(|native| native == module))
}

fn module_manifest() -> Result<&'static Vec<String>, String> {
    static MODULES: std::sync::OnceLock<Result<Vec<String>, String>> = std::sync::OnceLock::new();
    MODULES
        .get_or_init(|| serde_json::from_str(NATIVE_MODULES).map_err(|error| error.to_string()))
        .as_ref()
        .map_err(Clone::clone)
}

pub async fn invoke(
    module: &str,
    client: &Client,
    params: &Value,
    ip: IpAddr,
) -> Result<ModuleResponse, String> {
    match module {
        "ai_recommend" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/recommend")
                    .data(json!({
                        "platform": "ios",
                        "clientver": clientver,
                        "clienttime": clienttime,
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "client_playlist": [],
                        "source_type": 2,
                        "playlist_ver": 2,
                        "area_code": 1,
                        "appid": appid,
                        "key": sign_params_key(clienttime, is_lite),
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "recommend_source": csv_number_objects(params, "album_audio_id", "ID"),
                    }))
                    .header("x-router", "songlistairec.kugou.com")
                    .clear_default_params(),
            )
            .await
        }
        "album" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let mut data = Map::from_iter([
                ("appid".to_owned(), json!(appid)),
                ("clienttime".to_owned(), json!(clienttime)),
                ("clientver".to_owned(), json!(clientver)),
                (
                    "data".to_owned(),
                    Value::Array(
                        js_string(&value_or(params, "album_id", json!("")))
                            .split(',')
                            .map(|album_id| json!({ "album_id": album_id, "album_name": "", "author_name": "" }))
                            .collect(),
                    ),
                ),
                (
                    "dfid".to_owned(),
                    cookie_or_param(params, "dfid", json!("-")),
                ),
                ("fields".to_owned(), value_or(params, "fields", json!(""))),
                (
                    "key".to_owned(),
                    json!(sign_params_key(clienttime, is_lite)),
                ),
                (
                    "mid".to_owned(),
                    cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                ),
            ]);
            let token = cookie_or_param(params, "token", json!(0));
            let userid = cookie_or_param(params, "userid", json!(0));
            if truthy(&token) {
                data.insert("token".to_owned(), token);
            }
            if truthy(&userid) {
                data.insert("userid".to_owned(), userid);
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/album")
                    .base_url("http://kmr.service.kugou.com")
                    .data(Value::Object(data))
                    .header("x-router", "kmr.service.kugou.com")
                    .header("Content-Type", "application/json"),
            )
            .await
        }
        "album_detail" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/kmr/v2/albums")
                    .data(json!({
                        "data": [{ "album_id": value(params, "id") }],
                        "is_buy": value_or(params, "is_buy", json!(0)),
                        "fields": "album_id,album_name,publish_date,sizable_cover,intro,language,is_publish,heat,type,quality,authors,exclusive,author_name,trans_param",
                    }))
                    .header("x-router", "openapi.kugou.com")
                    .header("kg-tid", "255"),
            )
            .await
        }
        "album_shop" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/zhuanjidata/v3/album_shop_v2/get_classify_data"),
            )
            .await
        }
        "album_songs" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/album_audio/lite")
                    .data(json!({
                        "album_id": value(params, "id"),
                        "is_buy": value_or(params, "is_buy", json!("")),
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                    }))
                    .header("x-router", "openapi.kugou.com")
                    .header("kg-tid", "255"),
            )
            .await
        }
        "artist_albums" => {
            let sort = if params.get("sort").and_then(Value::as_str) == Some("hot") {
                3
            } else {
                1
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/kmr/v1/author/albums")
                    .data(json!({
                        "author_id": value(params, "id"),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "page": value_or(params, "page", json!(1)),
                        "sort": sort,
                        "category": 1,
                        "area_code": "all",
                    }))
                    .header("x-router", "openapi.kugou.com")
                    .header("kg-tid", "36"),
            )
            .await
        }
        "artist_audios" => {
            let clienttime = unix_time_millis()? / 1000;
            let (appid, clientver, is_lite) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/kmr/v1/audio_group/author")
                    .base_url("https://openapi.kugou.com")
                    .data(json!({
                        "appid": appid,
                        "clientver": clientver,
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "clienttime": clienttime,
                        "key": sign_params_key(clienttime, is_lite),
                        "author_id": value(params, "id"),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "page": value_or(params, "page", json!(1)),
                        "sort": if params.get("sort").and_then(Value::as_str) == Some("hot") { 1 } else { 2 },
                        "area_code": "all",
                    }))
                    .header("x-router", "openapi.kugou.com")
                    .header("kg-tid", "220"),
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
        "artist_follow_newsongs" => {
            let last_album_id = value_or(params, "last_album_id", json!(0));
            let opt_sort = if params.get("opt_sort").and_then(Value::as_i64) == Some(2) {
                2
            } else {
                1
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/feed/v1/follow/newsong_album_list")
                    .params(json!({
                        "last_album_id": last_album_id.clone(),
                        "page_size": value_or(params, "pagesize", json!(30)),
                        "opt_sort": opt_sort,
                    }))
                    .data(json!({ "last_album_id": last_album_id })),
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
        "artist_lists" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/ocean/v6/singer/list").params(json!({
                    "musician": number_value(value_or(params, "musician", json!(0))),
                    "sextype": value_or(params, "sextypes", json!(0)),
                    "showtype": 2,
                    "type": value_or(params, "type", json!(0)),
                    "hotsize": number_value(value_or(params, "hotsize", json!(30))),
                })),
            )
            .await
        }
        "artist_videos" => {
            let tag = match params.get("tag").and_then(Value::as_str).unwrap_or("all") {
                "official" => json!(18),
                "live" => json!(20),
                "fan" => json!(23),
                "artist" => json!(42419),
                _ => json!(""),
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/kmr/v1/author/videos")
                    .base_url("https://openapicdn.kugou.com")
                    .params(json!({
                        "author_id": value(params, "id"),
                        "is_fanmade": "",
                        "tag_idx": tag,
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "page": value_or(params, "page", json!(1)),
                    })),
            )
            .await
        }
        "captcha_sent" => {
            let mobile = params
                .get("mobile")
                .map(js_string)
                .unwrap_or_else(|| "undefined".to_owned());
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v7/send_mobile_code")
                    .base_url("http://login.user.kugou.com")
                    .without_cookie()
                    .data(json!({ "businessid": 5, "mobile": mobile, "plat": 3 })),
            )
            .await
        }
        "audio" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let mut data = Map::from_iter([
                ("appid".to_owned(), json!(appid)),
                ("clienttime".to_owned(), json!(clienttime)),
                ("clientver".to_owned(), json!(clientver)),
                ("data".to_owned(), csv_hash_audio(params)),
                (
                    "dfid".to_owned(),
                    cookie_or_param(params, "dfid", json!("-")),
                ),
                (
                    "key".to_owned(),
                    json!(sign_params_key(clienttime, is_lite)),
                ),
                (
                    "mid".to_owned(),
                    cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                ),
            ]);
            let token = cookie_or_param(params, "token", json!(0));
            let userid = cookie_or_param(params, "userid", json!(0));
            if truthy(&token) {
                data.insert("token".to_owned(), token);
            }
            if truthy(&userid) {
                data.insert("userid".to_owned(), userid);
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/audio/audio")
                    .base_url("http://kmr.service.kugou.com")
                    .data(Value::Object(data))
                    .header("x-router", "kmr.service.kugou.com")
                    .header("Content-Type", "application/json"),
            )
            .await
        }
        "audio_accompany_matching" => {
            let (appid, _, _) = platform_config();
            let mix_id = number_value(value(params, "mixId"));
            let mut query = Map::from_iter([
                ("isteen".to_owned(), json!(0)),
                (
                    "mixId".to_owned(),
                    if truthy(&mix_id) { mix_id } else { json!(0) },
                ),
                ("usemkv".to_owned(), json!(1)),
                ("platform".to_owned(), json!(2)),
                (
                    "fileName".to_owned(),
                    value_or(params, "fileName", json!("")),
                ),
                ("hash".to_owned(), value(params, "hash")),
                ("version".to_owned(), json!(12375)),
                ("appid".to_owned(), json!(appid)),
            ]);
            let signature = format!(
                "{:x}",
                md5::compute(format!("{}*s&iN#G70*", sorted_pairs(&query, "&")))
            );
            query.insert("sign".to_owned(), json!(&signature[8..24]));
            android_request(
                client,
                params,
                ip,
                NativeRequest::get(
                    "/sing7/accompanywan/json/v2/cdn/optimal_matching_accompany_2_listen.do",
                )
                .base_url("https://nsongacsing.kugou.com")
                .params(Value::Object(query))
                .clear_default_params()
                .unsigned(),
            )
            .await
        }
        "audio_ktv_total" => {
            let (appid, _, _) = platform_config();
            let mut query = Map::from_iter([
                ("isteen".to_owned(), json!(0)),
                (
                    "songId".to_owned(),
                    number_value(value(params, "songId")),
                ),
                ("usemkv".to_owned(), json!(1)),
                ("platform".to_owned(), json!(2)),
                ("singerName".to_owned(), value(params, "singerName")),
                ("songHash".to_owned(), value(params, "songHash")),
                ("version".to_owned(), json!(12375)),
                ("appid".to_owned(), json!(appid)),
            ]);
            let signature = format!(
                "{:x}",
                md5::compute(format!("{}*s&iN#G70*", sorted_pairs(&query, "&")))
            );
            query.insert("sign".to_owned(), json!(&signature[8..24]));
            android_request(
                client,
                params,
                ip,
                NativeRequest::get(
                    "/sing7/listenguide/json/v2/cdn/listenguide/get_total_opus_num_v02.do",
                )
                .base_url("https://acsing.service.kugou.com")
                .params(Value::Object(query))
                .clear_default_params()
                .unsigned(),
            )
            .await
        }
        "audio_related" => {
            let show_detail = number_is_zero(params.get("show_detail"));
            let mut query = Map::from_iter([
                (
                    "album_audio_id".to_owned(),
                    number_value(value(params, "album_audio_id")),
                ),
                ("appid".to_owned(), json!(1005)),
                ("area_code".to_owned(), json!(1)),
                ("clientver".to_owned(), json!(12329)),
            ]);
            if !show_detail {
                let sort = match params.get("sort").and_then(Value::as_str) {
                    Some("hot") => 2,
                    Some("new") => 3,
                    _ => 1,
                };
                query.extend(Map::from_iter([
                    ("page".to_owned(), value_or(params, "page", json!(1))),
                    (
                        "pagesize".to_owned(),
                        value_or(params, "pagesize", json!(30)),
                    ),
                    ("show_input".to_owned(), json!(1)),
                    (
                        "show_type".to_owned(),
                        value_or(params, "show_type", json!(0)),
                    ),
                    ("sort".to_owned(), json!(sort)),
                    ("type".to_owned(), value_or(params, "type", json!(0))),
                ]));
            }
            query.insert("version".to_owned(), json!(1));
            let salt = "OIlwieks28dk2k092lksi2UIkp";
            query.insert(
                "signature".to_owned(),
                json!(format!(
                    "{:x}",
                    md5::compute(format!("{salt}{}{salt}", sorted_pairs(&query, "")))
                )),
            );
            android_request(
                client,
                params,
                ip,
                NativeRequest::get(if show_detail {
                    "/v2/audio_related/total"
                } else {
                    "/v3/album_audio/related"
                })
                .base_url("https://listkmrp3cdnretry.kugou.com")
                .params(Value::Object(query))
                .clear_default_params(),
            )
            .await
        }
        "comment_album" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/m.comment.service/v1/cmtlist").params(json!({
                    "childrenid": value(params, "id"),
                    "need_show_image": 1,
                    "p": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "show_classify": value_or(params, "show_classify", json!(1)),
                    "show_hotword_list": value_or(params, "show_hotword_list", json!(1)),
                    "code": "94f1792ced1df89aa68a7939eaf2efca",
                })),
            )
            .await
        }
        "comment_count" => {
            let mut query = Map::from_iter([
                ("r".to_owned(), json!("comments/getcommentsnum")),
                (
                    "code".to_owned(),
                    json!("fc4be23b4e972707f36b8a828a93ba8a"),
                ),
            ]);
            if let Some(hash) = params.get("hash").filter(|value| truthy(value)) {
                query.insert("hash".to_owned(), hash.clone());
            } else if let Some(id) = params.get("special_id").filter(|value| truthy(value)) {
                query.insert("childrenid".to_owned(), id.clone());
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/index.php")
                    .params(Value::Object(query))
                    .web()
                    .header("x-router", "sum.comment.service.kugou.com"),
            )
            .await
        }
        "comment_music" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/mcomment/v1/cmtlist").params(json!({
                    "mixsongid": value(params, "mixsongid"),
                    "need_show_image": 1,
                    "p": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "show_classify": value_or(params, "show_classify", json!(1)),
                    "show_hotword_list": value_or(params, "show_hotword_list", json!(1)),
                    "extdata": "0",
                    "code": "fc4be23b4e972707f36b8a828a93ba8a",
                })),
            )
            .await
        }
        "comment_music_classify" => {
            let sort = params
                .get("sort")
                .cloned()
                .filter(|value| number_value(value.clone()).as_f64() == Some(2.0))
                .unwrap_or_else(|| json!(1));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/mcomment/v1/cmt_classify_list").params(json!({
                    "mixsongid": value(params, "mixsongid"),
                    "need_show_image": 1,
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "type_id": value(params, "type_id"),
                    "extdata": "0",
                    "code": "fc4be23b4e972707f36b8a828a93ba8a",
                    "sort_method": sort,
                })),
            )
            .await
        }
        "comment_music_hotword" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/mcomment/v1/get_hot_word").params(json!({
                    "mixsongid": value(params, "mixsongid"),
                    "need_show_image": 1,
                    "p": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "hot_word": value(params, "hot_word"),
                    "extdata": "0",
                    "code": "fc4be23b4e972707f36b8a828a93ba8a",
                })),
            )
            .await
        }
        "comment_playlist" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/m.comment.service/v1/cmtlist").params(json!({
                    "childrenid": value(params, "id"),
                    "need_show_image": 1,
                    "p": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "show_classify": value_or(params, "show_classify", json!(1)),
                    "show_hotword_list": value_or(params, "show_hotword_list", json!(1)),
                    "code": "ca53b96fe5a1d9c22d71c8f522ef7c4f",
                    "content_type": 0,
                    "tag": 5,
                })),
            )
            .await
        }
        "everyday_history" => {
            let mut query = Map::from_iter([
                (
                    "mode".to_owned(),
                    value_or(params, "mode", json!("list")),
                ),
                (
                    "platform".to_owned(),
                    value_or(params, "platform", json!("ios")),
                ),
            ]);
            for name in ["history_name", "date"] {
                if let Some(value) = params.get(name).filter(|value| truthy(value)) {
                    query.insert(name.to_owned(), value.clone());
                }
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/everyday/api/v1/get_history")
                    .params(Value::Object(query))
                    .header("x-router", "everydayrec.service.kugou.com"),
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
        "get_model" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/ocean/v6/sound/list").params(json!({
                    "super_vip": 1,
                    "sound_ver": 2,
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "apiver": 3,
                    "classify": "2,3",
                    "plat": 2,
                    "privilege": 1,
                    "sort": 2,
                })),
            )
            .await
        }
        "get_mode_info" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis();
            let is_lite = std::env::var("platform").as_deref() == Ok("lite");
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/api/v5/earphone/get_model_info")
                    .base_url("http://mobileservice.kugou.com")
                    .params(json!({
                        "model_id": number_value(value_or(params, "model_id", json!(0))),
                        "req_src": "collection",
                        "earphone_vip": 1,
                        "sound_ver": 2,
                        "key": sign_params_key(now, is_lite),
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                    })),
            )
            .await
        }
        "get_verify_info" => {
            let userid = number_value(param_or_cookie(params, "userid", json!("0")));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/verifyservice/v3/get_verify_info").data(json!({
                    "eventid": value(params, "eventid"),
                    "userid": userid,
                    "platid": value_or(params, "platid", json!(2)),
                    "rtype": 1,
                    "wasm": 1,
                    "i": "",
                    "sid": "",
                    "edt": "",
                })),
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
        "ip" => {
            let kind = match params.get("type").and_then(Value::as_str) {
                Some("albums") => "albums",
                Some("videos") => "videos",
                Some("author_list") => "author_list",
                _ => "audios",
            };
            let url = match kind {
                "albums" => "/openapi/v1/ip/albums",
                "videos" => "/openapi/v1/ip/videos",
                "author_list" => "/openapi/v1/ip/author_list",
                _ => "/openapi/v1/ip/audios",
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::post(url).data(json!({
                    "is_publish": 1,
                    "ip_id": value(params, "id"),
                    "sort": 3,
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "query": 1,
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
        "kmr_audio_mv" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/kmr/v1/audio/mv")
                    .data(json!({
                        "data": csv_objects(params, "album_audio_id", "album_audio_id"),
                        "fields": value_or(params, "fields", json!("")),
                    }))
                    .header("x-router", "openapi.kugou.com")
                    .header("KG-TID", "38"),
            )
            .await
        }
        "krm_audio" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/kmr/v2/audio")
                    .data(json!({
                        "data": csv_number_objects(params, "album_audio_id", "entity_id"),
                        "fields": value_or(params, "fields", json!("base")),
                    }))
                    .header("x-router", "openapi.kugou.com")
                    .header("KG-TID", "238"),
            )
            .await
        }
        "longaudio_album_audios" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/longaudio/v2/album_audios")
                    .data(json!({
                        "album_id": value(params, "album_id"),
                        "area_code": 1,
                        "tagid": 0,
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                    }))
                    .header("x-router", "openapi.kugou.com")
                    .header("KG-TID", "78"),
            )
            .await
        }
        "longaudio_album_detail" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/openapi/v2/broadcast")
                    .data(json!({
                        "data": csv_objects(params, "album_id", "album_id"),
                        "show_album_tag": 1,
                        "fields": "album_name,album_id,category,authors,sizable_cover,intro,author_name,trans_param,album_tag,mix_intro,full_intro,is_publish",
                    }))
                    .header("KG-TID", "78"),
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
        "login_qr_key" => {
            let (config_appid, _, _) = platform_config();
            let appid = if params.get("type").and_then(Value::as_str) == Some("web") {
                1014
            } else {
                1001
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v2/qrcode")
                    .base_url("https://login-user.kugou.com")
                    .params(json!({
                        "appid": appid,
                        "type": 1,
                        "plat": 4,
                        "qrcode_txt": format!("https://h5.kugou.com/apps/loginQRCode/html/index.html?appid={config_appid}&"),
                        "srcappid": 2919,
                    }))
                    .web(),
            )
            .await
        }
        "login_qr_check" => {
            let mut response = android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v2/get_userinfo_qrcode")
                    .base_url("https://login-user.kugou.com")
                    .params(json!({
                        "plat": 4,
                        "appid": platform_config().0,
                        "srcappid": 2919,
                        "qrcode": value(params, "key"),
                    }))
                    .web(),
            )
            .await?;
            if response
                .body
                .pointer("/data/status")
                .is_some_and(|value| number_value(value.clone()).as_i64() == Some(4))
            {
                if let Some(token) = response.body.pointer("/data/token") {
                    response.cookie.push(format!("token={}", js_string(token)));
                }
                if let Some(userid) = response.body.pointer("/data/userid") {
                    response
                        .cookie
                        .push(format!("userid={}", js_string(userid)));
                }
            }
            Ok(response)
        }
        "login_wx_check" => login_wx_check(client, params).await,
        "lastest_songs_listen" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/playque/devque/v1/get_latest_songs").data(json!({
                    "area_code": "1",
                    "sources": ["pc", "mobile", "tv", "car"],
                    "userid": number_value(param_or_cookie(params, "userid", json!(0))),
                    "ret_info": 1,
                    "token": param_or_cookie(params, "token", json!("")),
                    "pagesize": number_value(value_or(params, "pagesize", json!(30))),
                })),
            )
            .await
        }
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
        "playlist_detail" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v3/get_list_info")
                    .data(json!({
                        "data": csv_objects(params, "ids", "global_collection_id"),
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "token": param_or_cookie(params, "token", json!("")),
                    }))
                    .header("x-router", "pubsongs.kugou.com"),
            )
            .await
        }
        "playlist_similar" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis();
            let (appid, clientver, is_lite) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/pubsongs/v1/kmr_get_similar_lists").data(json!({
                    "appid": appid,
                    "clientver": clientver,
                    "clienttime": now,
                    "key": sign_params_key(now, is_lite),
                    "userid": param_or_cookie(params, "userid", json!(0)),
                    "ugc": 1,
                    "show_list": 1,
                    "need_songs": 1,
                    "data": csv_objects(params, "ids", "global_collection_id"),
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
        "playlist_tracks_del" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v4/delete_songs")
                    .data(json!({
                        "listid": value(params, "listid"),
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "data": csv_number_objects(params, "fileids", "fileid"),
                        "type": 0,
                        "token": param_or_cookie(params, "token", json!("")),
                        "list_ver": 0,
                    }))
                    .header("x-router", "cloudlist.service.kugou.com"),
            )
            .await
        }
        "playhistory_upload" => {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_secs();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/playhistory/v1/upload_songs")
                    .params(json!({ "plat": 3 }))
                    .data(json!({
                        "songs": [{
                            "mxid": number_value(value(params, "mxid")),
                            "op": 1,
                            "ot": number_value(value_or(params, "time", json!(now))),
                            "pc": number_value(value_or(params, "pc", json!(1))),
                        }],
                        "token": param_or_cookie(params, "token", json!("")),
                        "userid": param_or_cookie(params, "userid", json!(0)),
                    })),
            )
            .await
        }
        "recommend_songs" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/everyday_song_recommend")
                    .data(json!({
                        "platform": value_or(params, "platform", json!("android")),
                        "userid": param_or_cookie(params, "userid", json!("0")),
                    }))
                    .header("x-router", "everydayrec.service.kugou.com"),
            )
            .await
        }
        "rank_info" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/ocean/v6/rank/info").params(json!({
                    "rank_cid": value_or(params, "rank_cid", json!(0)),
                    "rankid": value(params, "rankid"),
                    "with_album_img": value_or(params, "album_img", json!(1)),
                    "zone": value_or(params, "zone", json!("")),
                })),
            )
            .await
        }
        "rank_audio" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/openapi/kmr/v2/rank/audio")
                    .data(json!({
                        "show_portrait_mv": 1,
                        "show_type_total": 1,
                        "filter_original_remarks": 1,
                        "area_code": 1,
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "rank_cid": value_or(params, "rank_cid", json!(0)),
                        "type": 1,
                        "page": value_or(params, "page", json!(1)),
                        "rank_id": value(params, "rankid"),
                    }))
                    .header("kg-tid", "369"),
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
        "scene_audio_list" => {
            let (appid, clientver, _) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/scene/v1/scene/audio_list")
                    .params(json!({
                        "scene_id": value(params, "id"),
                        "module_id": value(params, "module_id"),
                        "tag": value(params, "tag"),
                        "page": value_or(params, "page", json!(1)),
                        "page_size": value_or(params, "pagesize", json!(30)),
                    }))
                    .data(json!({
                        "appid": appid,
                        "clientver": clientver,
                        "token": param_or_cookie(params, "token", json!("")),
                        "userid": param_or_cookie(params, "userid", json!(0)),
                    })),
            )
            .await
        }
        "scene_collection_list" => {
            let (appid, clientver, _) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/scene/v1/distribution/collection_list").data(json!({
                    "appid": appid,
                    "clientver": clientver,
                    "token": param_or_cookie(params, "token", json!("")),
                    "userid": param_or_cookie(params, "userid", json!(0)),
                    "tag_id": value(params, "tag_id"),
                    "page": value_or(params, "page", json!(1)),
                    "page_size": value_or(params, "pagesize", json!(30)),
                    "exposed_data": [],
                })),
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
        "scene_lists_v2" => {
            let sort = match params.get("sort").and_then(Value::as_str).unwrap_or("rec") {
                "hot" => 2,
                "new" => 3,
                _ => 1,
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/scene/v1/scene/list_v2")
                    .params(json!({
                        "scene_id": value(params, "id"),
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "sort_type": sort,
                        "kugouid": param_or_cookie(params, "userid", json!("0")),
                    }))
                    .data(json!({ "exposure": [] })),
            )
            .await
        }
        "scene_video_list" => {
            let (appid, clientver, _) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/scene/v1/distribution/video_list").data(json!({
                    "appid": appid,
                    "clientver": clientver,
                    "token": param_or_cookie(params, "token", json!("")),
                    "userid": param_or_cookie(params, "userid", json!(0)),
                    "tag_id": value(params, "tag_id"),
                    "page": value_or(params, "page", json!(1)),
                    "page_size": value_or(params, "pagesize", json!(30)),
                    "exposed_data": [],
                })),
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
        "search_lyric" => {
            let (appid, clientver, _) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/search")
                    .base_url("https://lyrics.kugou.com")
                    .clear_default_params()
                    .params(json!({
                        "album_audio_id": value_or(params, "album_audio_id", json!(0)),
                        "appid": appid,
                        "clientver": clientver,
                        "duration": value_or(params, "duration", json!(0)),
                        "hash": value_or(params, "hash", json!("")),
                        "keyword": value_or(params, "keywords", json!("")),
                        "lrctxt": 1,
                        "man": value_nullish(params, "man", json!("no")),
                    })),
            )
            .await
        }
        "search_complex" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v6/search/complex")
                    .base_url("https://complexsearch.kugou.com")
                    .params(json!({
                        "platform": "AndroidFilter",
                        "keyword": value(params, "keywords"),
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "cursor": 0,
                    })),
            )
            .await
        }
        "search_suggest" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v2/getSearchTip")
                    .params(json!({
                        "keyword": value(params, "keywords"),
                        "AlbumTipCount": value_or(params, "albumTipCount", json!(10)),
                        "CorrectTipCount": value_or(params, "correctTipCount", json!(10)),
                        "MVTipCount": value_or(params, "mvTipCount", json!(10)),
                        "MusicTipCount": value_or(params, "musicTipCount", json!(10)),
                        "radiotip": 1,
                    }))
                    .header("x-router", "searchtip.kugou.com"),
            )
            .await
        }
        "server_now" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/server_now")
                    .params(json!({ "plat": 3 }))
                    .data(json!({
                        "token": param_or_cookie(params, "token", json!("")),
                        "userid": param_or_cookie(params, "userid", json!(0)),
                    }))
                    .header("x-router", "usercenter.kugou.com"),
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
        "sheet_explore" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/opern/v1/home/get_rec_opern")
                    .params(json!({
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "page": value_or(params, "page", json!(1)),
                        "opern_level": value_or(params, "level", json!(0)),
                        "instruments": value_or(params, "instruments", json!(1)),
                        "tagid": value_or(params, "tagid", json!(0)),
                    }))
                    .data(json!({ "exposure_mixids": "" })),
            )
            .await
        }
        "sheet_rank" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/opern/v1/home/get_rank_opern").params(json!({
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "page": value_or(params, "page", json!(1)),
                    "opern_level": value_or(params, "level", json!(0)),
                    "instruments": value_or(params, "instruments", json!(1)),
                    "tagid": value_or(params, "tagid", json!(0)),
                })),
            )
            .await
        }
        "sheet_song" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/opern/v1/detail/song_info").params(json!({
                    "mixsongid": value(params, "album_audio_id"),
                    "instruments": value_nullish(params, "instruments", json!(1)),
                    "opern_level": value_nullish(params, "level", json!(0)),
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
        "song_url" => {
            let is_lite = platform_config().2;
            let quality = params
                .get("quality")
                .filter(|value| truthy(value))
                .map(js_string)
                .map(|quality| {
                    if ["piano", "acappella", "subwoofer", "ancient", "dj", "surnay"]
                        .contains(&quality.as_str())
                    {
                        json!(format!("magic_{quality}"))
                    } else {
                        json!(quality)
                    }
                })
                .unwrap_or_else(|| json!(128));
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v5/url")
                    .params(json!({
                        "album_id": number_value(value_nullish(params, "album_id", json!(0))),
                        "area_code": 1,
                        "hash": js_string(&value_or(params, "hash", json!(""))).to_lowercase(),
                        "ssa_flag": "is_fromtrack",
                        "version": 11430,
                        "page_id": if is_lite { 967177915 } else { 151369488 },
                        "quality": quality,
                        "album_audio_id": number_value(value_nullish(params, "album_audio_id", json!(0))),
                        "behavior": "play",
                        "pid": if is_lite { 411 } else { 2 },
                        "cmd": 26,
                        "pidversion": 3001,
                        "IsFreePart": u8::from(params.get("free_part").is_some_and(truthy)),
                        "ppage_id": if is_lite {
                            value_or(params, "ppage_id", json!("356753938,823673182,967485191"))
                        } else {
                            json!("463467626,350369493,788954147")
                        },
                        "cdnBackup": 1,
                        "module": "",
                        "clientver": 11430,
                    }))
                    .encrypt_key()
                    .header("x-router", "trackercdn.kugou.com"),
            )
            .await
        }
        "song_url_new" => {
            let (appid, _, _) = platform_config();
            let clienttime = unix_time_millis()?;
            let userid = number_value(param_or_cookie(params, "userid", json!("0")));
            let hash = params
                .get("hash")
                .filter(|value| truthy(value))
                .or_else(|| params.get("file_hash").filter(|value| truthy(value)))
                .or_else(|| params.get("FileHash").filter(|value| truthy(value)))
                .map(js_string)
                .unwrap_or_default();
            let mid = cookie_value(params, "KUGOU_API_MID")
                .map(|value| js_string(&value))
                .unwrap_or_else(|| "undefined".to_owned());
            let tracker_key = format!(
                "{:x}",
                md5::compute(format!(
                    "{hash}185672dd44712f60bb1736df5a377e82{appid}{mid}{}",
                    js_string(&userid)
                ))
            );
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v6/priv_url")
                    .base_url("http://tracker.kugou.com")
                    .data(json!({
                        "area_code": "1",
                        "behavior": "play",
                        "qualities": ["128", "320", "flac", "high", "multitrack", "viper_atmos", "viper_tape", "viper_clear", "super"],
                        "resource": {
                            "album_audio_id": value(params, "album_audio_id"),
                            "collect_list_id": "3",
                            "collect_time": clienttime,
                            "hash": hash,
                            "id": 0,
                            "page_id": 1,
                            "type": "audio",
                        },
                        "token": param_or_cookie(params, "token", json!("")),
                        "tracker_param": {
                            "all_m": 1,
                            "auth": "",
                            "is_free_part": u8::from(params.get("free_part").is_some_and(truthy)),
                            "key": tracker_key,
                            "module_id": 0,
                            "need_climax": 1,
                            "need_xcdn": 1,
                            "open_time": "",
                            "pid": "411",
                            "pidversion": "3001",
                            "priv_vip_type": "6",
                            "viptoken": param_or_cookie(params, "vip_token", json!("")),
                        },
                        "userid": js_string(&userid),
                        "vip": cookie_value(params, "vip_type")
                            .filter(truthy)
                            .unwrap_or_else(|| value_or(params, "vipType", json!(0))),
                    })),
            )
            .await
        }
        "theme_music" => {
            let clienttime = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_secs();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/everydayrec.service/v1/mul_theme_category_recommend").data(
                    json!({
                        "platform": "android",
                        "clienttime": clienttime,
                        "show_theme_category_ids": value(params, "ids"),
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "module_id": 508,
                    }),
                ),
            )
            .await
        }
        "theme_music_detail" => {
            let clienttime = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_secs();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/everydayrec.service/v1/theme_category_recommend").data(
                    json!({
                        "platform": "android",
                        "clienttime": clienttime,
                        "theme_category_id": value(params, "id"),
                        "show_theme_category_id": 0,
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "module_id": 508,
                    }),
                ),
            )
            .await
        }
        "theme_playlist" => {
            let (_, clientver, _) = platform_config();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v2/getthemelist")
                    .data(json!({
                        "platform": "android",
                        "clientver": clientver,
                        "clienttime": now,
                        "area_code": 1,
                        "module_id": 1,
                        "userid": param_or_cookie(params, "userid", json!(0)),
                    }))
                    .header("x-router", "everydayrec.service.kugou.com"),
            )
            .await
        }
        "theme_playlist_track" => {
            let (_, clientver, _) = platform_config();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| error.to_string())?
                .as_millis();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v2/gettheme_songidlist")
                    .data(json!({
                        "platform": "android",
                        "clientver": clientver,
                        "clienttime": now,
                        "area_code": 1,
                        "module_id": 1,
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "theme_id": value(params, "theme_id"),
                    }))
                    .header("x-router", "everydayrec.service.kugou.com"),
            )
            .await
        }
        "top_album" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/musicadservice/v1/mobile_newalbum_sp").data(json!({
                    "apiver": 20,
                    "token": param_or_cookie(params, "token", json!("")),
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "withpriv": 1,
                })),
            )
            .await
        }
        "top_song" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/musicadservice/container/v1/newsong_publish").data(json!({
                    "rank_id": value_or(params, "type", json!(21608)),
                    "userid": param_or_cookie(params, "userid", json!(0)),
                    "page": value_or(params, "page", json!(1)),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "tags": [],
                })),
            )
            .await
        }
        "top_tag_card_youth" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/v1/song/tag_card_recommend")
                    .params(json!({
                        "ver": "v2",
                        "area_code": 1,
                        "platform": "ios",
                        "module_id": 1,
                        "clientver": 11490,
                    }))
                    .data(json!({ "tagid": "", "u_info": "", "source_mixsong": "" })),
            )
            .await
        }
        "user_follow_message" => {
            let userid = param_or_cookie(params, "userid", json!("0"));
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/msg.mobile/v3/msgtag/history").params(json!({
                    "filter": 1,
                    "maxid": 0,
                    "pagesize": value_nullish(params, "pagesize", json!(30)),
                    "tag": format!("chat:{}_{}", js_string(&userid), js_string(&value(params, "id"))),
                })),
            )
            .await
        }
        "user_history" => {
            let mut data = Map::from_iter([
                (
                    "token".to_owned(),
                    param_or_cookie(params, "token", json!("")),
                ),
                (
                    "userid".to_owned(),
                    param_or_cookie(params, "userid", json!(0)),
                ),
                ("source_classify".to_owned(), json!("app")),
                ("to_subdivide_sr".to_owned(), json!(1)),
            ]);
            if let Some(bp) = params.get("bp").filter(|value| truthy(value)) {
                data.insert("bp".to_owned(), bp.clone());
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/playhistory/v1/get_songs").data(Value::Object(data)),
            )
            .await
        }
        "user_playlist" => {
            let userid = cookie_or_param(params, "userid", json!(0));
            let token = cookie_or_param(params, "token", json!(""));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v7/get_all_list")
                    .params(json!({
                        "plat": 1,
                        "userid": number_value(userid.clone()),
                        "token": token.clone(),
                    }))
                    .data(json!({
                        "userid": userid,
                        "token": token,
                        "total_ver": 979,
                        "type": 2,
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                    }))
                    .header("x-router", "cloudlist.service.kugou.com"),
            )
            .await
        }
        "user_purchased_albums" => {
            let (appid, clientver, _) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/openapi/v1/copyright/get_album_goods").data(json!({
                    "appid": appid,
                    "userid": number_value(cookie_value(params, "userid").unwrap_or_else(|| json!(0))),
                    "token": cookie_value(params, "token").unwrap_or_else(|| json!("")),
                    "page": number_value(value_or(params, "page", json!(1))),
                    "pagesize": number_value(value_or(params, "pagesize", json!(15))),
                    "clientver": clientver.to_string(),
                    "deleted": 0,
                })),
            )
            .await
        }
        "user_purchased_songs" => {
            let (appid, clientver, _) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/openapi/copyright/v1/audio/get_goods").data(json!({
                    "appid": appid,
                    "userid": number_value(cookie_value(params, "userid").unwrap_or_else(|| json!(0))),
                    "token": cookie_value(params, "token").unwrap_or_else(|| json!("")),
                    "page": number_value(value_or(params, "page", json!(1))),
                    "pagesize": number_value(value_or(params, "pagesize", json!(50))),
                    "clientver": clientver.to_string(),
                    "deleted": 0,
                    "need_audio_info": 1,
                    "area_code": "1",
                })),
            )
            .await
        }
        "user_video_collect" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/collectservice/v2/collect_list_mixvideo")
                    .params(json!({ "plat": 1 }))
                    .data(json!({
                        "userid": param_or_cookie(params, "userid", json!("0")),
                        "token": param_or_cookie(params, "token", json!("")),
                        "page": value_nullish(params, "page", json!(1)),
                        "pagesize": value_nullish(params, "pagesize", json!(30)),
                    })),
            )
            .await
        }
        "user_video_love" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/m.comment.service/v1/get_user_like_video").params(json!({
                    "kugouid": param_or_cookie(params, "userid", json!("0")),
                    "pagesize": value_nullish(params, "pagesize", json!(30)),
                    "load_video_info": 1,
                    "p": 1,
                    "plat": 1,
                })),
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
        "video_url" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v2/interface/index")
                    .params(json!({
                        "backupdomain": 1,
                        "cmd": 123,
                        "ext": "mp4",
                        "ismp3": 0,
                        "hash": value(params, "hash"),
                        "pid": 1,
                        "type": 1,
                    }))
                    .encrypt_key()
                    .header("x-router", "trackermv.kugou.com"),
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
        "youth_channel_similar" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/v1/channel/get_friendly_channel")
                    .params(json!({ "channel_id": value(params, "channel_id") }))
                    .data(json!({
                        "area_code": 1,
                        "playlist_ver": 2,
                        "vip_type": param_or_cookie(params, "vip_type", json!(0)),
                        "platform": "ios",
                    })),
            )
            .await
        }
        "youth_channel_song" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/api/channel/v1/channel_get_song_audit_passed").params(
                    json!({
                        "global_collection_id": value(params, "global_collection_id"),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "page": value_or(params, "page", json!(1)),
                        "is_filter": 0,
                    }),
                ),
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
        "youth_day_vip" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/v1/recharge/receive_vip_listen_song")
                    .params(json!({
                        "source_id": 90139,
                        "receive_day": value(params, "receive_day"),
                    }))
                    .header("content-type", "application/x-www-form-urlencoded"),
            )
            .await
        }
        "youth_day_vip_upgrade" => {
            let userid = number_value(param_or_cookie(params, "userid", json!(0)));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/v1/listen_song/upgrade_vip_reward")
                    .params(json!({ "kugouid": userid, "ad_type": 1 })),
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
        "youth_listen_song" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/v2/report/listen_song")
                    .params(json!({ "clientver": 10566 }))
                    .data(json!({
                        "mixsongid": value_or(params, "mixsongid", json!(666075191)),
                    }))
                    .header(
                        "user-agent",
                        "Android13-1070-10566-201-0-ReportPlaySongToServerProtocol-wifi",
                    )
                    .header("content-type", "application/json; charset=utf-8"),
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
        "youth_user_song" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/youth/v1/get_user_song_public").params(json!({
                    "filter_video": 0,
                    "type": value_or(params, "type", json!(0)),
                    "userid": value(params, "userid"),
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "page": value_or(params, "page", json!(1)),
                    "is_filter": 0,
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
        "yueku_banner" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/ads.gateway/v3/listen_banner").data(json!({
                    "plat": 0,
                    "channel": 201,
                    "operator": 7,
                    "networktype": 2,
                    "userid": param_or_cookie(params, "userid", json!(0)),
                    "vip_type": 0,
                    "m_type": 0,
                    "tags": [],
                    "apiver": 5,
                    "ability": 2,
                    "mode": "normal",
                })),
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
        "everyday_friend" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post(
                    "/sing7/relation/json/v3/friend_rec_by_using_song_list",
                )
                .base_url("https://acsing.service.kugou.com")
                .params(json!({ "channel": 130, "isteen": 0, "platform": 2, "usemkv": 1 }))
                .data(json!({
                    "list": [{
                        "user_id": 853927886,
                        "mixsong_ids": [290083753,251724346,571554587,250126644,208831644,40328518,250504076,581706850,318347675,585258401,288481998,407414475,28239430,280584633,291957521,64556644,243149863,488725103,32114153,39951172,29019580,40397606,327507651,32029382,32218359,340353127,276448762,177071956,100031397,249251602]
                    }]
                }))
                .header("pid", "126556797"),
            )
            .await
        }
        "fm_class" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let userid = cookie_or_param(params, "userid", json!(0));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/class_fm_song")
                    .data(json!({
                        "kguid": userid,
                        "clienttime": clienttime,
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "platform": "android",
                        "clientver": clientver,
                        "uid": userid,
                        "get_tracker": 1,
                        "key": sign_params_key(clienttime, is_lite),
                        "appid": appid,
                    }))
                    .header("x-router", "fm.service.kugou.com"),
            )
            .await
        }
        "fm_image" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let mut data = Map::from_iter([
                ("appid".to_owned(), json!(appid)),
                ("clienttime".to_owned(), json!(clienttime)),
                ("clientver".to_owned(), json!(clientver)),
                (
                    "data".to_owned(),
                    Value::Array(
                        js_string(&value_or(params, "fmid", json!("")))
                            .split(',')
                            .map(|fmid| json!({ "fields": "imgUrl100,imgUrl50", "fmid": fmid, "fmtype": 2 }))
                            .collect(),
                    ),
                ),
                (
                    "dfid".to_owned(),
                    cookie_or_param(params, "dfid", json!("-")),
                ),
                (
                    "key".to_owned(),
                    json!(sign_params_key(clienttime, is_lite)),
                ),
                (
                    "mid".to_owned(),
                    cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                ),
            ]);
            let userid = cookie_or_param(params, "userid", Value::Null);
            let token = cookie_or_param(params, "token", Value::Null);
            if truthy(&userid) {
                data.insert("userid".to_owned(), userid);
            }
            if truthy(&token) {
                data.insert("token".to_owned(), token);
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/fm_info")
                    .data(Value::Object(data))
                    .header("x-router", "fm.service.kugou.com")
                    .header("Content-Type", "application/json"),
            )
            .await
        }
        "fm_recommend" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/rcmd_list")
                    .data(json!({
                        "appid": appid,
                        "clientver": clientver,
                        "clienttime": clienttime,
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "key": sign_params_key(clienttime, is_lite),
                        "rcmdsongcount": 1,
                        "level": 0,
                        "area_code": 1,
                        "get_tracker": 1,
                        "uid": 0,
                    }))
                    .header("x-router", "fm.service.kugou.com"),
            )
            .await
        }
        "fm_songs" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let mut fm_data: Vec<Value> = js_string(&value_or(params, "fmid", json!("")))
                .split(',')
                .map(|fmid| {
                    json!({
                        "fmid": fmid,
                        "fmtype": value_or(params, "type", json!(2)),
                        "offset": value_or(params, "offset", json!(-1)),
                        "size": value_or(params, "size", json!(20)),
                        "singername": "",
                    })
                })
                .collect();
            for (name, field) in [
                ("fmtype", "fmtype"),
                ("fmoffset", "offset"),
                ("fmsize", "size"),
            ] {
                for (index, item) in js_string(&value_or(params, name, json!("")))
                    .split(',')
                    .enumerate()
                {
                    if let Some(object) = fm_data.get_mut(index).and_then(Value::as_object_mut)
                        && !item.is_empty()
                    {
                        object.insert(field.to_owned(), json!(item));
                    }
                }
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/app_song_list_offset")
                    .data(json!({
                        "appid": appid,
                        "area_code": 1,
                        "clienttime": clienttime,
                        "clientver": clientver,
                        "data": fm_data,
                        "get_tracker": 1,
                        "key": sign_params_key(clienttime, is_lite),
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "uid": cookie_or_param(params, "userid", Value::Null),
                    }))
                    .header("x-router", "fm.service.kugou.com")
                    .header("Content-Type", "application/json"),
            )
            .await
        }
        "personal_fm" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let userid = cookie_or_param(params, "userid", json!(0));
            let token = cookie_or_param(params, "token", json!(0));
            let vip_type = cookie_or_param(params, "vip_type", value_or(params, "vipType", json!(0)));
            let mut data = Map::from_iter([
                ("appid".to_owned(), json!(appid)),
                ("clienttime".to_owned(), json!(clienttime)),
                (
                    "mid".to_owned(),
                    cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                ),
                ("action".to_owned(), value_or(params, "action", json!("play"))),
                ("recommend_source_locked".to_owned(), json!(0)),
                (
                    "song_pool_id".to_owned(),
                    number_value(value_or(params, "song_pool_id", json!(0))),
                ),
                ("callerid".to_owned(), json!(0)),
                ("m_type".to_owned(), json!(1)),
                (
                    "platform".to_owned(),
                    value_or(params, "platform", json!("ios")),
                ),
                ("area_code".to_owned(), json!(1)),
                (
                    "remain_songcnt".to_owned(),
                    number_value(value_or(params, "remain_songcnt", json!(0))),
                ),
                ("clientver".to_owned(), json!(clientver)),
                (
                    "is_overplay".to_owned(),
                    json!(u8::from(params.get("is_overplay").is_some_and(truthy))),
                ),
                ("mode".to_owned(), value_or(params, "mode", json!("normal"))),
                (
                    "fakem".to_owned(),
                    json!("ca981cfc583a4c37f28d2d49000013c16a0a"),
                ),
                (
                    "key".to_owned(),
                    json!(sign_params_key(clienttime, is_lite)),
                ),
            ]);
            if truthy(&userid) {
                data.insert("userid".to_owned(), userid.clone());
                data.insert("kguid".to_owned(), userid);
            }
            if truthy(&token) {
                data.insert("token".to_owned(), token);
            }
            if truthy(&vip_type) {
                data.insert("vip_type".to_owned(), vip_type);
            }
            for field in ["hash", "songid", "playtime"] {
                if let Some(value) = params.get(field).filter(|value| truthy(value)) {
                    data.insert(field.to_owned(), value.clone());
                }
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v2/personal_recommend")
                    .data(Value::Object(data))
                    .header("x-router", "persnfm.service.kugou.com"),
            )
            .await
        }
        "playlist_add" => {
            let userid = param_or_cookie(params, "userid", json!(0));
            let token = param_or_cookie(params, "token", json!(""));
            let list_type = value_or(params, "type", json!(0));
            let mut data = Map::from_iter([
                ("userid".to_owned(), userid.clone()),
                ("token".to_owned(), token.clone()),
                ("total_ver".to_owned(), json!(0)),
                ("name".to_owned(), value(params, "name")),
                ("type".to_owned(), list_type.clone()),
                (
                    "source".to_owned(),
                    if number_is_zero(params.get("source")) {
                        json!(0)
                    } else {
                        value_or(params, "source", json!(1))
                    },
                ),
                ("is_pri".to_owned(), json!(0)),
                (
                    "list_create_userid".to_owned(),
                    value(params, "list_create_userid"),
                ),
                (
                    "list_create_listid".to_owned(),
                    value(params, "list_create_listid"),
                ),
                (
                    "list_create_gid".to_owned(),
                    value_or(params, "list_create_gid", json!("")),
                ),
                ("from_shupinmv".to_owned(), json!(0)),
            ]);
            if number_is_zero(Some(&list_type)) {
                data.insert("is_pri".to_owned(), value_or(params, "is_pri", json!(0)));
            }
            let query = if number_is_zero(Some(&list_type)) {
                json!({
                    "last_time": unix_time_millis()? / 1000,
                    "last_area": "gztx",
                    "userid": userid,
                    "token": token,
                })
            } else {
                json!({})
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/cloudlist.service/v5/add_list")
                    .params(query)
                    .data(Value::Object(data)),
            )
            .await
        }
        "playlist_track_all" => {
            let pagesize = number_value(value_or(params, "pagesize", json!(30)))
                .as_f64()
                .unwrap_or(30.0);
            let page = number_value(value_or(params, "page", json!(1)))
                .as_f64()
                .unwrap_or(1.0);
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/pubsongs/v2/get_other_list_file_nofilt").params(json!({
                    "area_code": 1,
                    "begin_idx": (page - 1.0) * pagesize,
                    "plat": 1,
                    "type": 1,
                    "mode": 1,
                    "personal_switch": 1,
                    "extend_fields": "abtags,hot_cmt,popularization",
                    "pagesize": value_or(params, "pagesize", json!(30)),
                    "global_collection_id": value(params, "id"),
                })),
            )
            .await
        }
        "playlist_track_all_new" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v4/get_list_all_file")
                    .data(json!({
                        "listid": value(params, "listid"),
                        "userid": param_or_cookie(params, "userid", json!("0")),
                        "area_code": 1,
                        "show_relate_goods": 0,
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "allplatform": 1,
                        "show_cover": 1,
                        "type": 0,
                        "token": param_or_cookie(params, "token", json!("0")),
                        "page": value_or(params, "page", json!(1)),
                    }))
                    .header("x-router", "cloudlist.service.kugou.com"),
            )
            .await
        }
        "playlist_tracks_add" => {
            let userid = param_or_cookie(params, "userid", json!(0));
            let token = param_or_cookie(params, "token", json!(""));
            let resource = js_string(&value_or(params, "data", json!("")))
                .split(',')
                .map(|item| {
                    let fields: Vec<_> = item.split('|').collect();
                    json!({
                        "number": 1,
                        "name": fields.first().copied().unwrap_or(""),
                        "hash": fields.get(1).copied().unwrap_or(""),
                        "size": 0,
                        "sort": 0,
                        "timelen": 0,
                        "bitrate": 0,
                        "album_id": fields.get(2).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                        "mixsongid": fields.get(3).copied().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                    })
                })
                .collect::<Vec<_>>();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/cloudlist.service/v6/add_song")
                    .params(json!({
                        "last_time": unix_time_millis()? / 1000,
                        "last_area": "gztx",
                        "userid": userid,
                        "token": token,
                    }))
                    .data(json!({
                        "userid": userid,
                        "token": token,
                        "listid": value(params, "listid"),
                        "list_ver": 0,
                        "type": 0,
                        "slow_upload": 1,
                        "scene": "false;null",
                        "data": resource,
                    })),
            )
            .await
        }
        "privilege_lite" => {
            let (appid, clientver, _) = platform_config();
            let mut resource = csv_hash_resource(params, "hash", "hash", json!({
                "type": "audio",
                "page_id": 0,
                "album_id": 0,
            }));
            apply_csv_field(&mut resource, params, "album_id", "album_id");
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v2/get_res_privilege/lite")
                    .data(json!({
                        "appid": appid,
                        "area_code": 1,
                        "behavior": "play",
                        "clientver": clientver,
                        "need_hash_offset": 1,
                        "relate": 1,
                        "support_verify": 1,
                        "resource": resource,
                        "qualities": ["128", "320", "flac", "high", "viper_atmos", "viper_tape", "viper_clear", "super", "multitrack"],
                    }))
                    .header("x-router", "media.store.kugou.com")
                    .header("Content-Type", "application/json"),
            )
            .await
        }
        "search" => {
            let search_type = match params.get("type").and_then(Value::as_str) {
                Some("special") => "special",
                Some("lyric") => "lyric",
                Some("album") => "album",
                Some("author") => "author",
                Some("mv") => "mv",
                _ => "song",
            };
            let url = match search_type {
                "song" => "/v3/search/song",
                "special" => "/v1/search/special",
                "lyric" => "/v1/search/lyric",
                "album" => "/v1/search/album",
                "author" => "/v1/search/author",
                _ => "/v1/search/mv",
            };
            android_request(
                client,
                params,
                ip,
                NativeRequest::get(url)
                    .params(json!({
                        "albumhide": 0,
                        "iscorrection": 1,
                        "keyword": value_or(params, "keywords", json!("")),
                        "nocollect": 0,
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "platform": "AndroidFilter",
                    }))
                    .header("x-router", "complexsearch.kugou.com"),
            )
            .await
        }
        "search_default" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/searchnofocus/v1/search_no_focus_word")
                    .params(json!({ "clientver": 12329 }))
                    .data(json!({
                        "plat": 0,
                        "userid": number_value(param_or_cookie(params, "userid", json!(0))),
                        "tags": "{}",
                        "vip_type": param_or_cookie(params, "vip_type", json!(65530)),
                        "m_type": 0,
                        "own_ads": {},
                        "ability": "3",
                        "sources": [],
                        "bitmap": 2,
                        "mode": "normal",
                    })),
            )
            .await
        }
        "search_mixed" => {
            let clienttime = unix_time_millis()?;
            let requestid = format!(
                "{:x}_0",
                md5::compute(format!("bdaa53d04e7475feb9024164a47032f9{clienttime}"))
            );
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v3/search/mixed")
                    .params(json!({
                        "ab_tag": 0,
                        "ability": 511,
                        "albumhide": 0,
                        "apiver": 22,
                        "area_code": 1,
                        "clientver": 20125,
                        "cursor": 0,
                        "is_gpay": 0,
                        "iscorrection": 1,
                        "keyword": value(params, "keyword"),
                        "nocollect": 0,
                        "osversion": 16.5,
                        "platform": "IOSFilter",
                        "recver": 2,
                        "req_ai": 1,
                        "requestid": requestid,
                        "search_ability": 3,
                        "sec_aggre": 1,
                        "sec_aggre_bitmap": 0,
                        "style_type": 3,
                        "tag": "em",
                    }))
                    .header("x-router", "complexsearch.kugou.com")
                    .header("kg-clienttimems", clienttime.to_string()),
            )
            .await
        }
        "top_card" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let fakem = "60f7ebf1f812edbac3c63a7310001701760f";
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/singlecardrec.service/v1/single_card_recommend")
                    .params(json!({
                        "card_id": value_or(params, "card_id", json!(1)),
                        "fakem": fakem,
                        "area_code": 1,
                        "platform": "ios",
                    }))
                    .data(json!({
                        "appid": appid,
                        "clientver": clientver,
                        "platform": "android",
                        "clienttime": clienttime,
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "key": sign_params_key(clienttime, is_lite),
                        "fakem": fakem,
                        "area_code": 1,
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "uuid": "-",
                        "client_playlist": [],
                        "u_info": "a0c35cd40af564444b5584c2754dedec",
                    })),
            )
            .await
        }
        "top_card_youth" => {
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/youth/v1/song/single_card_recommend")
                    .params(json!({
                        "card_id": value_or(params, "card_id", json!(3005)),
                        "area_code": 1,
                        "platform": "ios",
                        "module_id": 1,
                        "ver": "v2",
                        "pagesize": value_nullish(params, "pagesize", json!(30)),
                        "clientver": 11490,
                    }))
                    .data(json!({
                        "tagid": value_nullish(params, "tagid", json!("")),
                        "u_info": "",
                        "source_mixsong": "",
                    })),
            )
            .await
        }
        "top_playlist" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = ((unix_time_millis()? as f64 / 1000.0).round() as u64).to_string();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v2/special_recommend")
                    .data(json!({
                        "appid": appid,
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "clientver": clientver,
                        "platform": "android",
                        "clienttime": clienttime,
                        "userid": param_or_cookie(params, "userid", json!(0)),
                        "module_id": value_or(params, "module_id", json!(1)),
                        "page": value_or(params, "page", json!(1)),
                        "pagesize": value_or(params, "pagesize", json!(30)),
                        "key": sign_params_key(&clienttime, is_lite),
                        "special_recommend": {
                            "withtag": value_or(params, "withtag", json!(1)),
                            "withsong": value_or(params, "withsong", json!(1)),
                            "sort": value_or(params, "sort", json!(1)),
                            "ugc": 1,
                            "is_selected": 0,
                            "withrecommend": 1,
                            "area_code": 1,
                            "categoryid": value_or(params, "category_id", json!(0)),
                        },
                        "req_multi": 1,
                        "retrun_min": 5,
                        "return_special_falg": 1,
                    }))
                    .header("x-router", "specialrec.service.kugou.com"),
            )
            .await
        }
        "video_detail" => {
            let (appid, clientver, is_lite) = platform_config();
            let clienttime = unix_time_millis()? / 1000;
            let dfid = cookie_value(params, "dfid").unwrap_or_else(|| json!("-"));
            let mid = cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null);
            let uuid = format!("{:x}", md5::compute(format!("{}{}", js_string(&dfid), js_string(&mid))));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/video")
                    .data(json!({
                        "appid": appid,
                        "clientver": clientver,
                        "clienttime": clienttime,
                        "mid": mid,
                        "uuid": uuid,
                        "dfid": dfid,
                        "token": param_or_cookie(params, "token", json!("")),
                        "key": sign_params_key(clienttime, is_lite),
                        "show_resolution": 1,
                        "data": csv_objects(params, "id", "video_id"),
                    }))
                    .clear_default_params()
                    .header("x-router", "kmr.service.kugou.com"),
            )
            .await
        }
        "video_privilege" => {
            let (appid, clientver, _) = platform_config();
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/get_video_privilege")
                    .data(json!({
                        "appid": appid,
                        "area_code": 1,
                        "behavior": "play",
                        "clientver": clientver,
                        "dfid": cookie_value(params, "dfid").unwrap_or_else(|| json!("-")),
                        "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                        "resource": Value::Array(
                            js_string(&value_or(params, "hash", json!("")))
                                .split(',')
                                .map(|hash| json!({ "hash": hash, "id": 0, "name": "" }))
                                .collect()
                        ),
                        "token": cookie_value(params, "token").unwrap_or_else(|| json!("")),
                        "userid": cookie_value(params, "userid").unwrap_or_else(|| json!(0)),
                        "vip": cookie_value(params, "vip_type").unwrap_or_else(|| json!(0)),
                    }))
                    .header("x-router", "media.store.kugou.com"),
            )
            .await
        }
        "brush" => {
            let (appid, _, is_lite) = platform_config();
            let clienttime = unix_time_millis()?;
            let userid = cookie_or_param(params, "userid", json!(0));
            let vip_type = cookie_value(params, "vip_type")
                .filter(truthy)
                .unwrap_or_else(|| value_or(params, "vipType", json!(0)));
            android_request(
                client,
                params,
                ip,
                NativeRequest::post("/genesisapi/v1/newepoch_song_rec/feed")
                    .params(json!({
                        "sort_type": 1,
                        "platform": "ios",
                        "page": 1,
                        "content_ver": 4,
                        "clientver": 11850,
                    }))
                    .data(json!({
                        "behaviors": [],
                        "abtest": { "abtest": { "shuashua": { "commentcard": 2 } } },
                        "personal_recommend_params": {
                            "userid": userid,
                            "appid": appid,
                            "playlist_ver": 2,
                            "clienttime": clienttime,
                            "mid": cookie_value(params, "KUGOU_API_MID").unwrap_or(Value::Null),
                            "new_sync_point": clienttime,
                            "module_id": 1,
                            "action": "login",
                            "vip_type": vip_type,
                            "vip_flags": 3,
                            "recommend_source_locked": 0,
                            "song_pool_id": number_value(value_or(params, "song_pool_id", json!(0))),
                            "callerid": 0,
                            "m_type": 1,
                            "kguid": userid,
                            "platform": "ios",
                            "area_code": 1,
                            "fakem": "ca981cfc583a4c37f28d2d49000013c16a0a",
                            "clientver": 11850,
                            "mode": value_or(params, "mode", json!("normal")),
                            "active_swtich": "on",
                            "key": sign_params_key(clienttime, is_lite),
                        },
                    })),
            )
            .await
        }
        "comment_floor" => {
            let resource_type = params
                .get("resource_type")
                .filter(|value| truthy(value))
                .or_else(|| params.get("resourceType").filter(|value| truthy(value)))
                .map(js_string)
                .unwrap_or_default()
                .to_ascii_lowercase();
            let song_code = "fc4be23b4e972707f36b8a828a93ba8a";
            let playlist_code = "ca53b96fe5a1d9c22d71c8f522ef7c4f";
            let album_code = "94f1792ced1df89aa68a7939eaf2efca";
            let code = params
                .get("code")
                .filter(|value| nonempty(value))
                .map(js_string)
                .unwrap_or_else(|| match resource_type.as_str() {
                    "playlist" => playlist_code.to_owned(),
                    "album" => album_code.to_owned(),
                    _ => song_code.to_owned(),
                });
            let service = matches!(resource_type.as_str(), "playlist" | "album")
                || code == playlist_code
                || code == album_code;
            let mut query = Map::from_iter([
                ("childrenid".to_owned(), value(params, "special_id")),
                ("need_show_image".to_owned(), json!(1)),
                ("p".to_owned(), value_or(params, "page", json!(1))),
                (
                    "pagesize".to_owned(),
                    value_or(params, "pagesize", json!(30)),
                ),
                (
                    "show_classify".to_owned(),
                    value_nullish(params, "show_classify", json!(1)),
                ),
                (
                    "show_hotword_list".to_owned(),
                    value_nullish(params, "show_hotword_list", json!(1)),
                ),
                ("code".to_owned(), json!(code)),
                ("tid".to_owned(), value(params, "tid")),
            ]);
            if let Some(mixsongid) = params.get("mixsongid").filter(|value| nonempty(value)) {
                query.insert("mixsongid".to_owned(), mixsongid.clone());
            }
            android_request(
                client,
                params,
                ip,
                NativeRequest::post(if service {
                    "/m.comment.service/v1/hot_replylist"
                } else {
                    "/mcomment/v1/hot_replylist"
                })
                .params(Value::Object(query)),
            )
            .await
        }
        "images" => {
            let (appid, clientver, is_lite) = platform_config();
            let mut data = csv_hash_resource(
                params,
                "hash",
                "hash",
                json!({ "album_id": 0, "album_audio_id": 0 }),
            );
            apply_csv_field_or_zero(&mut data, params, "album_id", "album_id");
            apply_csv_field_or_zero(&mut data, params, "album_audio_id", "album_audio_id");
            let query = Map::from_iter([
                ("album_image_type".to_owned(), json!("-3")),
                ("appid".to_owned(), json!(appid)),
                ("clientver".to_owned(), json!(clientver)),
                ("author_image_type".to_owned(), json!("3,4,5")),
                ("count".to_owned(), value_or(params, "count", json!(5))),
                ("data".to_owned(), Value::Array(data)),
                ("isCdn".to_owned(), json!(1)),
                ("publish_time".to_owned(), json!(1)),
            ]);
            let signature = signature_android(&query, "", is_lite);
            android_request(
                client,
                params,
                ip,
                NativeRequest::get(path_with_query("/container/v2/image", &query))
                    .base_url("https://expendablekmr.kugou.com")
                    .params(json!({ "signature": signature }))
                    .clear_default_params(),
            )
            .await
        }
        "images_audio" => {
            let (appid, clientver, is_lite) = platform_config();
            let mut data = csv_hash_resource(
                params,
                "hash",
                "hash",
                json!({ "audio_id": 0, "album_audio_id": 0, "filename": "" }),
            );
            apply_csv_field_or_zero(&mut data, params, "audio_id", "audio_id");
            apply_csv_field_or_zero(&mut data, params, "album_audio_id", "album_audio_id");
            apply_csv_field(&mut data, params, "filename", "filename");
            let query = Map::from_iter([
                ("appid".to_owned(), json!(appid)),
                ("clientver".to_owned(), json!(clientver)),
                ("count".to_owned(), value_or(params, "count", json!(5))),
                ("data".to_owned(), Value::Array(data)),
                ("isCdn".to_owned(), json!(1)),
                ("publish_time".to_owned(), json!(1)),
                ("show_authors".to_owned(), json!(1)),
            ]);
            let signature = signature_android(&query, "", is_lite);
            android_request(
                client,
                params,
                ip,
                NativeRequest::get(path_with_query("/v2/author_image/audio", &query))
                    .base_url("https://expendablekmr.kugou.com")
                    .params(json!({ "signature": signature }))
                    .clear_default_params(),
            )
            .await
        }
        "ip_zone" => {
            let mut response = android_request(
                client,
                params,
                ip,
                NativeRequest::get("/v1/zone/index")
                    .header("x-router", "yuekucategory.kugou.com"),
            )
            .await?;
            enrich_ip_zone(&mut response.body);
            Ok(response)
        }
        "top_ip" => {
            let mut response = android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v1/daily_recommend")
                    .base_url("http://musicadservice.kugou.com")
                    .params(json!({ "clientver": 12349, "area_code": 1 }))
                    .data(json!({ "tags": {} })),
            )
            .await?;
            enrich_top_ip(&mut response.body);
            Ok(response)
        }
        "user_cloud_url" => {
            let hash = params
                .get("hash")
                .map(js_string)
                .unwrap_or_else(|| "undefined".to_owned())
                .to_lowercase();
            let key = format!(
                "{:x}",
                md5::compute(format!(
                    "musicclound{hash}20026ebd1ac3134c880bda6a2194537843caa0162e2e7"
                ))
            );
            android_request(
                client,
                params,
                ip,
                NativeRequest::get("/bsstrackercdngz/v2/query_musicclound_url").params(json!({
                    "hash": hash,
                    "ssa_flag": "is_fromtrack",
                    "version": "20102",
                    "ssl": 0,
                    "album_audio_id": value_nullish(params, "album_audio_id", json!(0)),
                    "pid": 20026,
                    "audio_id": value_nullish(params, "audio_id", json!(0)),
                    "kv_id": 2,
                    "key": key,
                    "bucket": "musicclound",
                    "name": value_nullish(params, "name", json!("")),
                    "with_res_tag": 0,
                })),
            )
            .await
        }
        "user_cloud_match" => {
            let (default_appid, default_clientver, is_lite) = platform_config();
            let request_appid = value_or(params, "appid", json!(default_appid));
            let request_clientver = value_or(params, "clientver", json!(default_clientver));
            let clienttime = unix_time_millis()? / 1000;
            let hashes = split_values(value(params, "hash"));
            if hashes.is_empty() {
                return Ok(ModuleResponse {
                    status: 500,
                    body: json!({ "status": 0, "msg": "请传入 hash，或通过请求体传入文件二进制数据" }),
                    cookie: Vec::new(),
                    headers: HashMap::new(),
                });
            }
            let album_audio_ids = split_values(
                params
                    .get("album_audio_ids")
                    .filter(|value| truthy(value))
                    .or_else(|| params.get("album_audio_id").filter(|value| truthy(value)))
                    .or_else(|| params.get("mixid").filter(|value| truthy(value)))
                    .or_else(|| params.get("mix_id").filter(|value| truthy(value)))
                    .cloned()
                    .unwrap_or(Value::Null),
            );
            let data = hashes
                .iter()
                .enumerate()
                .map(|(index, hash)| {
                    let mut item = Map::from_iter([(
                        "hash".to_owned(),
                        json!(hash.to_ascii_lowercase()),
                    )]);
                    if let Some(album_audio_id) = album_audio_ids
                        .get(index)
                        .or_else(|| album_audio_ids.first())
                        .filter(|value| value.parse::<f64>().is_ok_and(|value| value > 0.0))
                    {
                        item.insert("album_audio_id".to_owned(), json!(album_audio_id));
                    }
                    Value::Object(item)
                })
                .collect::<Vec<_>>();
            let salt = if is_lite {
                "LnT6xpN3khm36zse0QzvmgTZ3waWdRSA"
            } else {
                "OIlwieks28dk2k092lksi2UIkp"
            };
            let key = format!(
                "{:x}",
                md5::compute(format!(
                    "{}{salt}{}{clienttime}",
                    js_string(&request_appid),
                    js_string(&request_clientver)
                ))
            );
            let mut response = android_request(
                client,
                params,
                ip,
                NativeRequest::post("/v2/album_audio/audio")
                    .base_url("http://kmr.service.kugou.com")
                    .data(json!({
                        "appid": request_appid,
                        "clienttime": clienttime,
                        "clientver": request_clientver,
                        "data": data,
                        "dfid": cookie_or_param(params, "dfid", json!("-")),
                        "key": key,
                        "mid": cookie_value(params, "KUGOU_API_MID")
                            .filter(truthy)
                            .unwrap_or_else(|| value_or(params, "mid", json!(""))),
                        "show_privilege": 0,
                        "show_author_alias": 0,
                        "show_rel_album_audio_info": 0,
                        "show_remarks": 0,
                    }))
                    .clear_default_params()
                    .unsigned()
                    .header("x-router", "kmr.service.kugou.com")
                    .header("Content-Type", "application/json"),
            )
            .await?;
            enrich_cloud_match(&mut response.body);
            Ok(response)
        }
        _ => Err(format!("unknown native module: {module}")),
    }
}

struct NativeRequest {
    url: String,
    method: Method,
    params: Map<String, Value>,
    data: Option<Value>,
    headers: HashMap<String, String>,
    base_url: String,
    signature: SignatureKind,
    encrypt_key: bool,
    use_input_cookie: bool,
    clear_default_params: bool,
}

#[derive(Clone, Copy)]
enum SignatureKind {
    Android,
    Web,
    None,
}

impl NativeRequest {
    fn get(url: impl Into<String>) -> Self {
        Self::new(url, Method::GET)
    }

    fn post(url: impl Into<String>) -> Self {
        Self::new(url, Method::POST)
    }

    fn delete(url: impl Into<String>) -> Self {
        Self::new(url, Method::DELETE)
    }

    fn new(url: impl Into<String>, method: Method) -> Self {
        Self {
            url: url.into(),
            method,
            params: Map::new(),
            data: None,
            headers: HashMap::new(),
            base_url: "https://gateway.kugou.com".to_owned(),
            signature: SignatureKind::Android,
            encrypt_key: false,
            use_input_cookie: true,
            clear_default_params: false,
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

    fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    fn web(mut self) -> Self {
        self.signature = SignatureKind::Web;
        self
    }

    fn unsigned(mut self) -> Self {
        self.signature = SignatureKind::None;
        self
    }

    fn encrypt_key(mut self) -> Self {
        self.encrypt_key = true;
        self
    }

    fn without_cookie(mut self) -> Self {
        self.use_input_cookie = false;
        self
    }

    fn clear_default_params(mut self) -> Self {
        self.clear_default_params = true;
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

fn cookie_value(params: &Value, name: &str) -> Option<Value> {
    params
        .get("cookie")
        .and_then(Value::as_object)
        .and_then(|cookie| cookie.get(name))
        .filter(|value| truthy(value))
        .cloned()
}

fn param_or_cookie(params: &Value, name: &str, default: Value) -> Value {
    params
        .get(name)
        .filter(|value| truthy(value))
        .cloned()
        .or_else(|| cookie_value(params, name))
        .unwrap_or(default)
}

fn cookie_or_param(params: &Value, name: &str, default: Value) -> Value {
    cookie_value(params, name)
        .or_else(|| params.get(name).filter(|value| truthy(value)).cloned())
        .unwrap_or(default)
}

fn number_value(value: Value) -> Value {
    match value {
        Value::Null => json!(0),
        Value::Bool(value) => json!(u8::from(value)),
        Value::Number(_) => value,
        Value::String(value) => value
            .trim()
            .parse::<f64>()
            .ok()
            .and_then(|value| {
                if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
                    Some(Value::Number(serde_json::Number::from(value as i64)))
                } else {
                    serde_json::Number::from_f64(value).map(Value::Number)
                }
            })
            .unwrap_or_else(|| Value::String("NaN".to_owned())),
        Value::Array(_) | Value::Object(_) => Value::String("NaN".to_owned()),
    }
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

fn csv_number_objects(params: &Value, name: &str, field: &str) -> Value {
    let values = params
        .get(name)
        .filter(|value| truthy(value))
        .map(js_string)
        .unwrap_or_default();
    Value::Array(
        values
            .split(',')
            .map(|value| {
                Value::Object(Map::from_iter([(
                    field.to_owned(),
                    number_value(Value::String(value.to_owned())),
                )]))
            })
            .collect(),
    )
}

fn csv_hash_audio(params: &Value) -> Value {
    let hashes = js_string(&value_or(params, "hash", json!("")));
    Value::Array(
        hashes
            .split(',')
            .map(|hash| json!({ "hash": hash, "audio_id": 0 }))
            .collect(),
    )
}

fn csv_hash_resource(params: &Value, name: &str, field: &str, base: Value) -> Vec<Value> {
    let values = js_string(&value_or(params, name, json!("")));
    values
        .split(',')
        .map(|value| {
            let mut object = base.as_object().cloned().unwrap_or_default();
            object.insert(field.to_owned(), json!(value));
            Value::Object(object)
        })
        .collect()
}

fn apply_csv_field(items: &mut [Value], params: &Value, name: &str, field: &str) {
    let values = js_string(&value_or(params, name, json!("")));
    for (index, value) in values.split(',').enumerate() {
        if let Some(object) = items.get_mut(index).and_then(Value::as_object_mut) {
            object.insert(field.to_owned(), json!(value));
        }
    }
}

fn apply_csv_field_or_zero(items: &mut [Value], params: &Value, name: &str, field: &str) {
    let values = js_string(&value_or(params, name, json!("")));
    for (index, value) in values.split(',').enumerate() {
        if let Some(object) = items.get_mut(index).and_then(Value::as_object_mut) {
            object.insert(
                field.to_owned(),
                if value.is_empty() {
                    json!(0)
                } else {
                    json!(value)
                },
            );
        }
    }
}

fn nonempty(value: &Value) -> bool {
    let value = js_string(value);
    let value = value.trim();
    !value.is_empty() && value != "null" && value != "undefined"
}

fn path_with_query(path: &str, params: &Map<String, Value>) -> String {
    let mut entries: Vec<_> = params.iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    for (name, value) in entries {
        query.append_pair(name, &js_string(value));
    }
    format!("{path}?{}", query.finish())
}

fn query_value(query: &str, name: &str) -> Option<String> {
    url::form_urlencoded::parse(query.trim_start_matches('?').as_bytes())
        .find_map(|(key, value)| (key == name).then(|| value.into_owned()))
}

fn split_values(value: Value) -> Vec<String> {
    match value {
        Value::Null => Vec::new(),
        Value::Array(values) => values
            .into_iter()
            .flat_map(|value| {
                js_string(&value)
                    .split(',')
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
            .collect(),
        value => js_string(&value)
            .split(',')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .collect(),
    }
}

fn enrich_cloud_match(body: &mut Value) {
    if body.get("status").and_then(Value::as_i64) != Some(1) {
        return;
    }
    let Some(data) = body.get("data").and_then(Value::as_array).cloned() else {
        return;
    };
    let matches = data
        .iter()
        .filter_map(|item| item.as_array().and_then(|items| items.first()).unwrap_or(item).as_object())
        .filter_map(|candidate| {
            let audio_info = candidate.get("audio_info").and_then(Value::as_object);
            let album_audio_id = number_value(
                candidate
                    .get("album_audio_id")
                    .cloned()
                    .unwrap_or(Value::Null),
            );
            let audio_id = number_value(
                audio_info
                    .and_then(|info| info.get("audio_id"))
                    .or_else(|| candidate.get("audio_id"))
                    .cloned()
                    .unwrap_or(Value::Null),
            );
            let hash = audio_info
                .and_then(|info| info.get("hash"))
                .filter(|value| truthy(value))
                .or_else(|| candidate.get("hash").filter(|value| truthy(value)))
                .cloned()
                .unwrap_or_else(|| json!(""));
            if number_is_zero(Some(&album_audio_id))
                && number_is_zero(Some(&audio_id))
                && !truthy(&hash)
            {
                return None;
            }
            let audio_name = ["ori_audio_name", "audio_name", "songname"]
                .into_iter()
                .find_map(|name| candidate.get(name).filter(|value| truthy(value)).cloned())
                .unwrap_or_else(|| json!(""));
            Some(json!({
                "album_audio_id": album_audio_id,
                "audio_id": audio_id,
                "hash_std": hash,
                "hash": hash,
                "author_name": candidate.get("author_name").filter(|value| truthy(value)).cloned().unwrap_or_else(|| json!("")),
                "audio_name": audio_name,
                "suffix_audio_name": candidate.get("suffix_audio_name").filter(|value| truthy(value)).cloned().unwrap_or_else(|| json!("")),
                "album_info": candidate.get("album_info").cloned().unwrap_or(Value::Null),
                "raw": Value::Object(candidate.clone()),
            }))
        })
        .collect::<Vec<_>>();
    if let Some(object) = body.as_object_mut() {
        object.insert("match_list".to_owned(), json!(matches));
        object.insert(
            "match".to_owned(),
            matches.first().cloned().unwrap_or(Value::Null),
        );
    }
}

fn enrich_ip_zone(body: &mut Value) {
    if body.get("status").and_then(Value::as_i64) != Some(1) {
        return;
    }
    let Some(list_value) = body
        .get_mut("data")
        .and_then(Value::as_object_mut)
        .and_then(|data| data.get_mut("list"))
        .filter(|value| truthy(value))
    else {
        return;
    };
    let mut list = list_value.as_array().cloned().unwrap_or_default();
    for item in &mut list {
        let Some(object) = item.as_object_mut() else {
            continue;
        };
        let Some(link) = object.get("special_link").filter(|value| truthy(value)) else {
            continue;
        };
        let Some(path) = query_value(&js_string(link), "path") else {
            continue;
        };
        let Some(ip_id) = query_value(&path, "ip_id") else {
            continue;
        };
        object.insert("ip_id".to_owned(), number_value(json!(ip_id)));
    }
    *list_value = Value::Array(list);
}

fn enrich_top_ip(body: &mut Value) {
    if body.get("status").and_then(Value::as_i64) != Some(1) {
        return;
    }
    let Some(list_value) = body
        .get_mut("data")
        .and_then(Value::as_object_mut)
        .and_then(|data| data.get_mut("list"))
    else {
        return;
    };
    let mut list = list_value.as_array().cloned().unwrap_or_default();
    for item in &mut list {
        let Some(extra) = item
            .as_object_mut()
            .and_then(|item| item.get_mut("extra"))
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        let Some(inner_url) = extra.get("inner_url").filter(|value| truthy(value)) else {
            continue;
        };
        let inner_url = js_string(inner_url);
        let Some(index) = inner_url.rfind("ip_id") else {
            continue;
        };
        let value = inner_url.get(index + 6..).unwrap_or_default();
        extra.insert("ip_id".to_owned(), number_value(json!(value)));
    }
    *list_value = Value::Array(list);
}

fn sorted_pairs(params: &Map<String, Value>, separator: &str) -> String {
    let mut entries: Vec<_> = params.iter().collect();
    entries.sort_by(|(left, _), (right, _)| left.cmp(right));
    entries
        .into_iter()
        .map(|(key, value)| format!("{key}={}", js_string(value)))
        .collect::<Vec<_>>()
        .join(separator)
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

fn unix_time_millis() -> Result<u64, String> {
    Ok(std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis() as u64)
}

fn platform_config() -> (u16, u32, bool) {
    let is_lite = std::env::var("platform").as_deref() == Ok("lite");
    if is_lite {
        (3116, 11440, true)
    } else {
        (1005, 20489, false)
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
    let cookie = if options.use_input_cookie {
        input
            .get("cookie")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    } else {
        Map::new()
    };
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
    let (appid, clientver, is_lite) = platform_config();

    let defaults = Map::from_iter([
        ("dfid".to_owned(), Value::String(dfid.clone())),
        ("mid".to_owned(), Value::String(mid.clone())),
        ("uuid".to_owned(), Value::String("-".to_owned())),
        ("appid".to_owned(), json!(appid)),
        ("clientver".to_owned(), json!(clientver)),
        ("clienttime".to_owned(), json!(clienttime)),
    ]);
    let mut query = if options.clear_default_params {
        Map::new()
    } else {
        defaults
    };
    if !options.clear_default_params && !token.is_empty() {
        query.insert("token".to_owned(), Value::String(token));
    }
    if !options.clear_default_params && truthy(&userid) {
        query.insert("userid".to_owned(), userid);
    }
    query.extend(options.params);

    if options.encrypt_key {
        let hash = query
            .get("hash")
            .map(js_string)
            .unwrap_or_else(|| "undefined".to_owned());
        let appid = query
            .get("appid")
            .filter(|value| truthy(value))
            .map(js_string)
            .unwrap_or_else(|| "1005".to_owned());
        let userid = query
            .get("userid")
            .filter(|value| truthy(value))
            .map(js_string)
            .unwrap_or_else(|| "0".to_owned());
        query.insert(
            "key".to_owned(),
            Value::String(sign_key(&hash, &mid, &userid, &appid, is_lite)),
        );
    }

    let data = options.data.as_ref().map_or_else(String::new, js_string);
    if !query.contains_key("signature") {
        let signature = match options.signature {
            SignatureKind::Android => Some(signature_android(&query, &data, is_lite)),
            SignatureKind::Web => Some(signature_web(&query)),
            SignatureKind::None => None,
        };
        if let Some(signature) = signature {
            query.insert("signature".to_owned(), Value::String(signature));
        }
    }

    let mut url = Url::parse(&options.base_url)
        .map_err(|error| error.to_string())?
        .join(&options.url)
        .map_err(|error| error.to_string())?;
    append_query(&mut url, &query);
    let mut request = client.request(options.method, url);
    for (name, value) in options.headers {
        request = request.header(name, value);
    }
    request = request
        .header(
            header::USER_AGENT,
            "Android15-1070-11083-46-0-DiscoveryDRADProtocol-wifi",
        )
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

fn sign_key(hash: &str, mid: &str, userid: &str, appid: &str, is_lite: bool) -> String {
    let salt = if is_lite {
        "185672dd44712f60bb1736df5a377e82"
    } else {
        "57ae12eb6890223e355ccfcb74edf70d"
    };
    format!(
        "{:x}",
        md5::compute(format!("{hash}{salt}{appid}{mid}{userid}"))
    )
}

fn sign_params_key(data: impl std::fmt::Display, is_lite: bool) -> String {
    let (appid, clientver, salt) = if is_lite {
        (3116, 11440, "LnT6xpN3khm36zse0QzvmgTZ3waWdRSA")
    } else {
        (1005, 20489, "OIlwieks28dk2k092lksi2UIkp")
    };
    format!(
        "{:x}",
        md5::compute(format!("{appid}{salt}{clientver}{data}"))
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
        assert_eq!(modules.len(), 145);
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

    #[test]
    fn encrypt_key_matches_node_vector() {
        assert_eq!(
            sign_key("ABC", "123", "0", "3116", true),
            "075554afd631fed34df32da5cceb7f24"
        );
    }

    #[test]
    fn params_key_matches_node_vector() {
        assert_eq!(
            sign_params_key(1700000000000u64, true),
            "d0ea4884a9807886a7c7e582cda713fa"
        );
    }

    #[test]
    fn js_number_conversion_keeps_integer_signature_format() {
        assert_eq!(js_string(&number_value(json!("1"))), "1");
        assert_eq!(js_string(&number_value(json!("1.5"))), "1.5");
    }
}
