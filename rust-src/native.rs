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
    encrypt_key: bool,
    use_input_cookie: bool,
    clear_default_params: bool,
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
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
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

fn number_is_zero(value: Option<&Value>) -> bool {
    match value {
        Some(Value::Null) => true,
        Some(Value::Bool(value)) => !value,
        Some(Value::Number(value)) => value.as_f64() == Some(0.0),
        Some(Value::String(value)) => value.trim().parse::<f64>() == Ok(0.0),
        _ => false,
    }
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
        assert_eq!(modules.len(), 110);
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
}
