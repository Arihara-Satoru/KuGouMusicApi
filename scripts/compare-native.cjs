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

function request(url) {
  return new Promise((resolve, reject) => {
    http.get(url, response => {
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
      { module: 'album_shop', route: '/album/shop', compareShape: true },
      { module: 'artist_detail', route: '/artist/detail?id=1', compareShape: true },
      { module: 'artist_honour', route: '/artist/honour?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'everyday_recommend', route: '/everyday/recommend?platform=ios', compareShape: true },
      { module: 'everyday_style_recommend', route: '/everyday/style/recommend?tagids=1', compareShape: true },
      { module: 'favorite_count', route: '/favorite/count?mixsongids=1', compareShape: true },
      { module: 'ip_dateil', route: '/ip/dateil?id=1', compareShape: true },
      { module: 'ip_playlist', route: '/ip/playlist?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'ip_zone_home', route: '/ip/zone/home?id=1', compareShape: true },
      { module: 'longaudio_daily_recommend', route: '/longaudio/daily/recommend?page=1&pagesize=2', compareShape: true },
      { module: 'longaudio_rank_recommend', route: '/longaudio/rank/recommend', compareShape: true },
      { module: 'longaudio_vip_recommend', route: '/longaudio/vip/recommend', compareShape: true },
      { module: 'longaudio_week_recommend', route: '/longaudio/week/recommend', compareShape: true },
      { module: 'login_qr_create', route: '/login/qr/create?key=contract-test' },
      { module: 'login_wx_check', route: '/login/wx/check?uuid=invalid-contract-test' },
      { module: 'pc_diantai', route: '/pc/diantai?userid=0', compareShape: true },
      { module: 'playlist_effect', route: '/playlist/effect?page=1&pagesize=2', compareShape: true },
      { module: 'playlist_tags', route: '/playlist/tags', compareShape: true },
      { module: 'rank_list', route: '/rank/list?withsong=1', compareShape: true },
      { module: 'rank_top', route: '/rank/top', compareShape: true },
      { module: 'rank_vol', route: '/rank/vol?rankid=1&rank_cid=0', compareShape: true },
      { module: 'scene_lists', route: '/scene/lists', compareShape: true },
      { module: 'scene_music', route: '/scene/music?id=1&page=1&pagesize=2', compareShape: true },
      { module: 'scene_module', route: '/scene/module?id=1', compareShape: true },
      { module: 'scene_module_info', route: '/scene/module/info?id=1&module_id=1', compareShape: true },
      { module: 'search_hot', route: '/search/hot', compareShape: true },
      { module: 'sheet_collection', route: '/sheet/collection?position=2', compareShape: true },
      { module: 'sheet_detail', route: '/sheet/detail?id=1', compareShape: true },
      { module: 'sheet_tags', route: '/sheet/tags', compareShape: true },
      { module: 'singer_list', route: '/singer/list?hotsize=2&sextype=0&type=0', compareShape: true },
      { module: 'song_climax', route: '/song/climax?hash=1', compareShape: true },
      { module: 'song_ranking', route: '/song/ranking?album_audio_id=1', compareShape: true },
      { module: 'song_ranking_filter', route: '/song/ranking/filter?album_audio_id=1&page=1&pagesize=2', compareShape: true },
      { module: 'user_vip_detail', route: '/user/vip/detail', compareShape: true },
      { module: 'youth_channel_all', route: '/youth/channel/all?page=1&pagesize=2', compareShape: true },
      { module: 'youth_channel_amway', route: '/youth/channel/amway?global_collection_id=1', compareShape: true },
      { module: 'youth_channel_detail', route: '/youth/channel/detail?global_collection_id=1', compareShape: true },
      { module: 'youth_channel_song_detail', route: '/youth/channel/song/detail?global_collection_id=1&fileid=1', compareShape: true },
      { module: 'youth_channel_sub', route: '/youth/channel/sub?global_collection_id=1&t=0', compareShape: true },
      { module: 'youth_dynamic', route: '/youth/dynamic', compareShape: true },
      { module: 'youth_dynamic_recent', route: '/youth/dynamic/recent', compareShape: true },
      { module: 'youth_month_vip_record', route: '/youth/month/vip/record', compareShape: true },
      { module: 'youth_union_vip', route: '/youth/union/vip', compareShape: true },
      { module: 'youth_vip', route: '/youth/vip', compareShape: true },
      { module: 'yueku', route: '/yueku', compareShape: true },
      { module: 'yueku_fm', route: '/yueku/fm', compareShape: true },
    ]
    assert.deepStrictEqual(cases.map(test => test.module).sort(), [...nativeModules].sort(), 'every native module needs a comparison case')
    for (const test of cases) {
      const [rustResponse, nodeResponse] = await Promise.all([
        request(`http://127.0.0.1:${rustPort}${test.route}`),
        request(`http://127.0.0.1:${nodePort}${test.route}`),
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
