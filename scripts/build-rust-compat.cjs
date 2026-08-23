#!/usr/bin/env node
const fs = require('fs')
const path = require('path')
const esbuild = require('esbuild')

const root = path.resolve(__dirname, '..')
const moduleDir = path.join(root, 'module')
const outputDir = path.join(root, 'rust-assets')
const temporaryEntry = path.join(outputDir, '.compat-entry.cjs')
const nativeModules = new Set(JSON.parse(fs.readFileSync(path.join(root, 'rust-native.json'), 'utf8')))

const allModules = fs.readdirSync(moduleDir)
  .filter(file => file.endsWith('.js') && !file.startsWith('_'))
  .sort()
for (const module of nativeModules) {
  if (!allModules.includes(`${module}.js`)) throw new Error(`native module does not exist: ${module}`)
}
const modules = allModules.filter(file => !nativeModules.has(file.slice(0, -3)))

fs.mkdirSync(outputDir, { recursive: true })
fs.writeFileSync(temporaryEntry, `
const { Buffer } = require('buffer')
globalThis.Buffer = Buffer
globalThis.process = globalThis.process || { env: {} }
globalThis.window = globalThis
globalThis.navigator = globalThis.navigator || { userAgent: 'kugoumusicapi-rust' }
globalThis.console = globalThis.console || { log() {}, warn() {}, error() {}, info() {}, debug() {} }
globalThis.URLSearchParams = globalThis.URLSearchParams || class URLSearchParams {
  constructor(values = {}) { this.values = Object.entries(values) }
  toString() { return this.values.map(([key, value]) => encodeURIComponent(key) + '=' + encodeURIComponent(value)).join('&') }
}

const { createRequest } = require('../util/request')
const modules = {
${modules.map(file => `  ${JSON.stringify(file.slice(0, -3))}: require(${JSON.stringify(`../module/${file}`)}),`).join('\n')}
}

function decode(value) {
  return JSON.parse(value, (_key, item) =>
    item && typeof item === 'object' && typeof item.__kugou_buffer__ === 'string'
      ? Buffer.from(item.__kugou_buffer__, 'base64')
      : item
  )
}

function encode(value) {
  return JSON.stringify(value, (_key, item) => {
    if (Buffer.isBuffer(item)) return { __kugou_buffer__: item.toString('base64') }
    if (item && item.type === 'Buffer' && Array.isArray(item.data)) {
      return { __kugou_buffer__: Buffer.from(item.data).toString('base64') }
    }
    if (item instanceof Error) return { message: item.message, stack: item.stack }
    return item
  })
}

globalThis.__kugou_set_env = value => {
  process.env = decode(value)
  // The Rust HTTP client owns proxy handling, so JS never needs Node's URL class.
  delete process.env.KUGOU_API_PROXY
}
globalThis.__kugou_modules = Object.keys(modules)
globalThis.__kugou_invoke = async (name, paramsJson, ip) => {
  const handler = modules[name]
  if (!handler) throw new Error('unknown module: ' + name)
  try {
    const response = await handler(decode(paramsJson), options => {
      options.ip = ip
      return createRequest(options)
    })
    return encode(response)
  } catch (error) {
    if (error && error.body) return encode(error)
    return encode({
      status: 404,
      body: { code: 404, data: null, msg: 'Not Found' },
      cookie: [],
      headers: {},
    })
  }
}
`)

const aliases = {
  axios: path.join(root, 'rust-compat', 'axios-shim.js'),
  crypto: path.join(root, 'rust-compat', 'crypto-shim.js'),
  'node:crypto': path.join(root, 'rust-compat', 'crypto-shim.js'),
  qrcode: path.join(root, 'rust-compat', 'qrcode-shim.js'),
}

async function main() {
  const output = path.join(outputDir, 'compat.js')
  await esbuild.build({
    entryPoints: [temporaryEntry],
    outfile: output,
    bundle: true,
    format: 'iife',
    platform: 'browser',
    target: 'es2020',
    minify: process.env.RUST_COMPAT_DEBUG !== '1',
    plugins: [{
      name: 'rust-compat-aliases',
      setup(build) {
        build.onResolve({ filter: /^(axios|crypto|node:crypto|qrcode)$/ }, args => ({ path: aliases[args.path] }))
      },
    }],
  })

  const bundle = fs.readFileSync(output, 'utf8')
    .replace(/[ \t]+$/gm, '')
    .replace(/^ +\t/gm, '\t')
    .replace(/\n+$/, '\n')
  fs.writeFileSync(output, bundle)
  fs.rmSync(temporaryEntry, { force: true })
  console.log(`[build-rust-compat] embedded ${modules.length} compatibility modules; ${nativeModules.size}/${allModules.length} native Rust modules`)
}

main().catch(error => {
  fs.rmSync(temporaryEntry, { force: true })
  console.error(error)
  process.exitCode = 1
})
