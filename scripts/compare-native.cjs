#!/usr/bin/env node
const assert = require('assert')
const { spawn } = require('child_process')
const http = require('http')
const path = require('path')
const nativeModules = require('../rust-native.json')

const root = path.resolve(__dirname, '..')
const rustPort = Number(process.env.COMPARE_RUST_PORT || 36630)
const nodePort = Number(process.env.COMPARE_NODE_PORT || 36631)
const rustBinary = process.env.COMPARE_RUST_BINARY || path.join(root, 'target', 'release', process.platform === 'win32' ? 'kugoumusicapi.exe' : 'kugoumusicapi')
const sharedEnv = {
  ...process.env,
  HOST: '127.0.0.1',
  platform: 'lite',
  KUGOU_API_GUID: '123e4567-e89b-42d3-a456-426614174000',
  KUGOU_API_DEV: 'CONTRACT01',
  KUGOU_API_MAC: '02:00:00:00:00:00',
  KUGOU_API_WEBGL: '123456789',
}

function start(command, args, port) {
  const child = spawn(command, args, {
    cwd: root,
    env: { ...sharedEnv, PORT: String(port) },
    stdio: ['ignore', 'pipe', 'pipe'],
    windowsHide: true,
  })
  let output = ''
  child.stdout.on('data', chunk => { output += chunk })
  child.stderr.on('data', chunk => { output += chunk })
  child.output = () => output
  return child
}

function request(url, options = {}) {
  return new Promise((resolve, reject) => {
    const body = options.body
    const request = http.request(url, {
      method: options.method || 'GET',
      headers: body ? { 'content-type': 'application/octet-stream', 'content-length': body.length } : {},
    }, response => {
      const chunks = []
      response.on('data', chunk => chunks.push(chunk))
      response.on('end', () => {
        const text = Buffer.concat(chunks).toString('utf8')
        try {
          resolve({ status: response.statusCode, body: JSON.parse(text) })
        } catch {
          resolve({ status: response.statusCode, body: text })
        }
      })
    }).on('error', reject)
    if (body) request.write(body)
    request.end()
  })
}

function shape(value) {
  if (value === null) return 'null'
  if (Array.isArray(value)) return 'array'
  if (typeof value !== 'object') return typeof value
  return Object.fromEntries(Object.keys(value).sort().map(key => [key, shape(value[key])]))
}

async function waitUntilReady(port, child) {
  const deadline = Date.now() + 15_000
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`server exited early:\n${child.output()}`)
    try {
      await request(`http://127.0.0.1:${port}/`)
      return
    } catch {
      await new Promise(resolve => setTimeout(resolve, 100))
    }
  }
  throw new Error(`server did not start:\n${child.output()}`)
}

async function main() {
  const rust = start(rustBinary, [], rustPort)
  const node = start(process.execPath, ['index.js'], nodePort)
  try {
    await Promise.all([waitUntilReady(rustPort, rust), waitUntilReady(nodePort, node)])
    const cases = [
      { module: 'ai_recommend', route: '/ai/recommend?album_audio_id=1&userid=0', compareShape: true },
      { module: 'album', route: '/album?album_id=1&fields=base', compareShape: true },
      { module: 'album_shop', route: '/album/shop', compareShape: true },
      { module: 'album_songs', route: '/album/songs?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'album_detail', route: '/album/detail?id=1', compareShape: true },
      { module: 'artist_albums', route: '/artist/albums?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'artist_audios', route: '/artist/audios?id=1&page=1&pagesize=2&sort=hot', compareShape: true },
      { module: 'artist_detail', route: '/artist/detail?id=1', compareShape: true },
      { module: 'artist_follow_newsongs', route: '/artist/follow/newsongs?last_album_id=0&pagesize=2', compareShape: true },
      { module: 'artist_honour', route: '/artist/honour?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'artist_lists', route: '/artist/lists?musician=0&hotsize=2', compareShape: true },
      { module: 'artist_videos', route: '/artist/videos?id=1&tag=all&page=1&pagesize=2', compareShape: true },
      { module: 'captcha_sent', route: '/captcha/sent', compareShape: true },
      { module: 'audio', route: '/audio?hash=ABC', compareShape: true },
      { module: 'audio_accompany_matching', route: '/audio/accompany/matching?mixId=1&fileName=test&hash=ABC', compareShape: true },
      { module: 'audio_ktv_total', route: '/audio/ktv/total?songId=1&singerName=test&songHash=ABC', compareShape: true },
      { module: 'audio_match', route: '/audio/match?userid=0', method: 'POST', body: Buffer.from('native-contract-audio'), compareShape: true },
      { module: 'audio_related', route: '/audio/related?album_audio_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'brush', route: '/brush?userid=0&song_pool_id=0', compareShape: true },
      { module: 'comment_floor', route: '/comment/floor?resource_type=song&special_id=1&mixsongid=1&tid=1&page=1&pagesize=2', compareShape: true },
      { module: 'comment_music_hotword', route: '/comment/music/hotword?mixsongid=1&hot_word=test&page=1&pagesize=2', compareShape: true },
      { module: 'comment_album', route: '/comment/album?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'comment_count', route: '/comment/count?hash=ABC', compareShape: true },
      { module: 'comment_music', route: '/comment/music?mixsongid=1&page=1&pagesize=2', compareShape: true },
      { module: 'comment_music_classify', route: '/comment/music/classify?mixsongid=1&type_id=1&sort=2', compareShape: true },
      { module: 'comment_playlist', route: '/comment/playlist?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'everyday_history', route: '/everyday/history?mode=list&platform=ios', compareShape: true },
      { module: 'everyday_friend', route: '/everyday/friend', compareShape: true },
      { module: 'everyday_recommend', route: '/everyday/recommend?platform=ios', compareShape: true },
      { module: 'everyday_style_recommend', route: '/everyday/style/recommend?tagids=1', compareShape: true },
      { module: 'favorite_count', route: '/favorite/count?mixsongids=1', compareShape: true },
      { module: 'fm_class', route: '/fm/class?userid=0', compareShape: true },
      { module: 'fm_image', route: '/fm/image?fmid=1', compareShape: true },
      { module: 'fm_recommend', route: '/fm/recommend', compareShape: true },
      { module: 'fm_songs', route: '/fm/songs?fmid=1&fmtype=2&fmoffset=-1&fmsize=2', compareShape: true },
      { module: 'get_model', route: '/get/model?page=1&pagesize=2', compareShape: true },
      { module: 'get_mode_info', route: '/get/mode/info?model_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'get_verify_info', route: '/get/verify/info?eventid=1&userid=0', compareShape: true },
      { module: 'ip_dateil', route: '/ip/dateil?id=1', compareShape: true },
      { module: 'ip', route: '/ip?id=1&type=audios&page=1&pagesize=2', compareShape: true },
      { module: 'ip_playlist', route: '/ip/playlist?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'ip_zone_home', route: '/ip/zone/home?id=1', compareShape: true },
      { module: 'ip_zone', route: '/ip/zone', compareShape: true },
      { module: 'images', route: '/images?hash=ABC&album_id=0&album_audio_id=0&count=1', compareShape: true },
      { module: 'images_audio', route: '/images/audio?hash=ABC&audio_id=0&album_audio_id=0&filename=test&count=1', compareShape: true },
      { module: 'longaudio_daily_recommend', route: '/longaudio/daily/recommend?page=1&pagesize=2', compareShape: true },
      { module: 'longaudio_album_audios', route: '/longaudio/album/audios?album_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'longaudio_album_detail', route: '/longaudio/album/detail?album_id=1', compareShape: true },
      { module: 'longaudio_rank_recommend', route: '/longaudio/rank/recommend', compareShape: true },
      { module: 'longaudio_vip_recommend', route: '/longaudio/vip/recommend', compareShape: true },
      { module: 'longaudio_week_recommend', route: '/longaudio/week/recommend', compareShape: true },
      { module: 'login_qr_create', route: '/login/qr/create?key=contract-test' },
      { module: 'login_qr_check', route: '/login/qr/check?key=invalid-contract-test', compareShape: true },
      { module: 'login_qr_key', route: '/login/qr/key?type=web', compareShape: true },
      { module: 'login_wx_check', route: '/login/wx/check?uuid=invalid-contract-test' },
      { module: 'kmr_audio_mv', route: '/kmr/audio/mv?album_audio_id=1', compareShape: true },
      { module: 'krm_audio', route: '/krm/audio?album_audio_id=1&fields=base', compareShape: true },
      { module: 'lastest_songs_listen', route: '/lastest/songs/listen?userid=0&pagesize=2', compareShape: true },
      { module: 'pc_diantai', route: '/pc/diantai?userid=0', compareShape: true },
      { module: 'personal_fm', route: '/personal/fm?userid=0&action=play', compareShape: true },
      { module: 'playlist_add', route: '/playlist/add?userid=0&type=1&name=test&list_create_userid=0&list_create_listid=0', compareShape: true },
      { module: 'playlist_effect', route: '/playlist/effect?page=1&pagesize=2', compareShape: true },
      { module: 'playlist_detail', route: '/playlist/detail?ids=1&userid=0', compareShape: true },
      { module: 'playlist_similar', route: '/playlist/similar?ids=1&userid=0', compareShape: true },
      { module: 'playlist_tags', route: '/playlist/tags', compareShape: true },
      { module: 'playlist_track_all', route: '/playlist/track/all?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'playlist_track_all_new', route: '/playlist/track/all/new?listid=1&userid=0&page=1&pagesize=2', compareShape: true },
      { module: 'playlist_tracks_add', route: '/playlist/tracks/add?listid=1&userid=0&data=test%7CABC%7C0%7C0', compareShape: true },
      { module: 'playlist_tracks_del', route: '/playlist/tracks/del?listid=1&fileids=1&userid=0', compareShape: true },
      { module: 'playhistory_upload', route: '/playhistory/upload?mxid=1&userid=0', compareShape: true },
      { module: 'privilege_lite', route: '/privilege/lite?hash=ABC&album_id=0', compareShape: true },
      { module: 'rank_audio', route: '/rank/audio?rankid=1&page=1&pagesize=2', compareShape: true },
      { module: 'recommend_songs', route: '/recommend/songs?platform=android&userid=0', compareShape: true },
      { module: 'rank_info', route: '/rank/info?rankid=1&rank_cid=0', compareShape: true },
      { module: 'rank_list', route: '/rank/list?withsong=1', compareShape: true },
      { module: 'rank_top', route: '/rank/top', compareShape: true },
      { module: 'rank_vol', route: '/rank/vol?rankid=1&rank_cid=0', compareShape: true },
      { module: 'scene_lists', route: '/scene/lists', compareShape: true },
      { module: 'scene_audio_list', route: '/scene/audio/list?id=1&module_id=1&tag=1&page=1&pagesize=2', compareShape: true },
      { module: 'scene_collection_list', route: '/scene/collection/list?tag_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'scene_music', route: '/scene/music?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'scene_lists_v2', route: '/scene/lists/v2?id=1&sort=rec&page=1&pagesize=2', compareShape: true },
      { module: 'scene_video_list', route: '/scene/video/list?tag_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'scene_module', route: '/scene/module?id=1', compareShape: true },
      { module: 'scene_module_info', route: '/scene/module/info?id=1&module_id=1', compareShape: true },
      { module: 'search_complex', route: '/search/complex?keywords=test&page=1&pagesize=2', compareShape: true },
      { module: 'search', route: '/search?keywords=test&type=song&page=1&pagesize=2', compareShape: true },
      { module: 'search_default', route: '/search/default?userid=0', compareShape: true },
      { module: 'search_mixed', route: '/search/mixed?keyword=test', compareShape: true },
      { module: 'search_hot', route: '/search/hot', compareShape: true },
      { module: 'search_lyric', route: '/search/lyric?album_audio_id=1&keywords=test', compareShape: true },
      { module: 'search_suggest', route: '/search/suggest?keywords=test', compareShape: true },
      { module: 'server_now', route: '/server/now?userid=0', compareShape: true },
      { module: 'sheet_collection', route: '/sheet/collection?position=2', compareShape: true },
      { module: 'sheet_explore', route: '/sheet/explore?page=1&pagesize=2', compareShape: true },
      { module: 'sheet_rank', route: '/sheet/rank?page=1&pagesize=2', compareShape: true },
      { module: 'sheet_song', route: '/sheet/song?album_audio_id=1', compareShape: true },
      { module: 'sheet_detail', route: '/sheet/detail?id=1', compareShape: true },
      { module: 'sheet_tags', route: '/sheet/tags', compareShape: true },
      { module: 'singer_list', route: '/singer/list?hotsize=2&sextype=0&type=0', compareShape: true },
      { module: 'song_climax', route: '/song/climax?hash=1', compareShape: true },
      { module: 'song_ranking', route: '/song/ranking?album_audio_id=1', compareShape: true },
      { module: 'song_ranking_filter', route: '/song/ranking/filter?album_audio_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'theme_music', route: '/theme/music?ids=1&userid=0', compareShape: true },
      { module: 'theme_music_detail', route: '/theme/music/detail?id=1&userid=0', compareShape: true },
      { module: 'theme_playlist', route: '/theme/playlist?userid=0', compareShape: true },
      { module: 'theme_playlist_track', route: '/theme/playlist/track?theme_id=1&userid=0', compareShape: true },
      { module: 'top_album', route: '/top/album?page=1&pagesize=2', compareShape: true },
      { module: 'top_card', route: '/top/card?card_id=1&userid=0', compareShape: true },
      { module: 'top_card_youth', route: '/top/card/youth?card_id=3005&pagesize=2', compareShape: true },
      { module: 'top_playlist', route: '/top/playlist?page=1&pagesize=2', compareShape: true },
      { module: 'top_ip', route: '/top/ip', compareShape: true },
      { module: 'top_song', route: '/top/song?type=21608&page=1&pagesize=2', compareShape: true },
      { module: 'top_tag_card_youth', route: '/top/tag/card/youth', compareShape: true },
      { module: 'user_follow_message', route: '/user/follow/message?userid=0&id=1&pagesize=2', compareShape: true },
      { module: 'user_history', route: '/user/history?userid=0', compareShape: true },
      { module: 'user_cloud_url', route: '/user/cloud/url?hash=ABC&album_audio_id=0&audio_id=0', compareShape: true },
      { module: 'user_cloud_match', route: '/user/cloud/match?hash=ABC&album_audio_id=0', compareShape: true },
      { module: 'user_playlist', route: '/user/playlist?userid=0&page=1&pagesize=2', compareShape: true },
      { module: 'user_purchased_albums', route: '/user/purchased/albums?page=1&pagesize=2', compareShape: true },
      { module: 'user_purchased_songs', route: '/user/purchased/songs?page=1&pagesize=2', compareShape: true },
      { module: 'user_video_collect', route: '/user/video/collect?userid=0&page=1&pagesize=2', compareShape: true },
      { module: 'user_video_love', route: '/user/video/love?userid=0&pagesize=2', compareShape: true },
      { module: 'user_vip_detail', route: '/user/vip/detail', compareShape: true },
      { module: 'video_url', route: '/video/url?hash=ABC', compareShape: true },
      { module: 'video_detail', route: '/video/detail?id=1', compareShape: true },
      { module: 'video_privilege', route: '/video/privilege?hash=ABC', compareShape: true },
      { module: 'youth_channel_all', route: '/youth/channel/all?page=1&pagesize=2', compareShape: true },
      { module: 'youth_channel_amway', route: '/youth/channel/amway?global_collection_id=1', compareShape: true },
      { module: 'youth_channel_detail', route: '/youth/channel/detail?global_collection_id=1', compareShape: true },
      { module: 'youth_channel_similar', route: '/youth/channel/similar?channel_id=1&vip_type=0', compareShape: true },
      { module: 'youth_channel_song', route: '/youth/channel/song?global_collection_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'youth_channel_song_detail', route: '/youth/channel/song/detail?global_collection_id=1&fileid=1', compareShape: true },
      { module: 'youth_channel_sub', route: '/youth/channel/sub?global_collection_id=1&t=0', compareShape: true },
      { module: 'youth_day_vip', route: '/youth/day/vip?receive_day=1', compareShape: true },
      { module: 'youth_day_vip_upgrade', route: '/youth/day/vip/upgrade?userid=0', compareShape: true },
      { module: 'youth_dynamic', route: '/youth/dynamic', compareShape: true },
      { module: 'youth_dynamic_recent', route: '/youth/dynamic/recent', compareShape: true },
      { module: 'youth_month_vip_record', route: '/youth/month/vip/record', compareShape: true },
      { module: 'youth_listen_song', route: '/youth/listen/song?mixsongid=666075191', compareShape: true },
      { module: 'youth_union_vip', route: '/youth/union/vip', compareShape: true },
      { module: 'youth_user_song', route: '/youth/user/song?userid=1&page=1&pagesize=2', compareShape: true },
      { module: 'youth_vip', route: '/youth/vip', compareShape: true },
      { module: 'yueku', route: '/yueku', compareShape: true },
      { module: 'yueku_banner', route: '/yueku/banner?userid=0', compareShape: true },
      { module: 'yueku_fm', route: '/yueku/fm', compareShape: true },
    ]
    assert.deepStrictEqual(cases.map(test => test.module).sort(), [...nativeModules].sort(), 'every native module needs a comparison case')
    for (const test of cases) {
      const [rustResponse, nodeResponse] = await Promise.all([
        request(`http://127.0.0.1:${rustPort}${test.route}`, test),
        request(`http://127.0.0.1:${nodePort}${test.route}`, test),
      ])
      for (const response of [rustResponse, nodeResponse]) {
        if (response.status === 502 && response.body?.status === 0 && response.body.msg) {
          response.body.msg = '<upstream-error>'
        }
      }
      if (test.compareShape) {
        rustResponse.body = shape(rustResponse.body)
        nodeResponse.body = shape(nodeResponse.body)
      }
      assert.deepStrictEqual(rustResponse, nodeResponse, test.route)
      console.log(`[compare-native] OK ${test.route}`)
    }
  } finally {
    rust.kill()
    node.kill()
  }
}

main().catch(error => {
  console.error(error)
  process.exitCode = 1
})
