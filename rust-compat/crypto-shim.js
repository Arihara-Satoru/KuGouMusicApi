const CryptoJS = require('crypto-js')
const forge = require('node-forge')

const toWordArray = (value) => {
  const bytes = Buffer.isBuffer(value) ? value : Buffer.from(value)
  return CryptoJS.enc.Hex.parse(bytes.toString('hex'))
}

const fromWordArray = (value) => Buffer.from(value.toString(CryptoJS.enc.Hex), 'hex')

function createHash(algorithm) {
  if (String(algorithm).toLowerCase() !== 'md5') throw new Error(`unsupported hash: ${algorithm}`)
  const chunks = []
  return {
    update(value) {
      chunks.push(Buffer.isBuffer(value) ? value : Buffer.from(value))
      return this
    },
    digest(encoding) {
      const value = fromWordArray(CryptoJS.MD5(toWordArray(Buffer.concat(chunks))))
      return encoding ? value.toString(encoding) : value
    },
  }
}

const createPublicKey = (pem) => pem

function publicEncrypt(options, value) {
  const pem = typeof options === 'string' ? options : options.key
  const key = forge.pki.publicKeyFromPem(pem)
  const encrypted = key.encrypt(Buffer.from(value).toString('binary'), 'RAW')
  return Buffer.from(encrypted, 'binary')
}

function randomBytes(size) {
  const bytes = Buffer.alloc(Number(size) || 0)
  for (let index = 0; index < bytes.length; index += 1) bytes[index] = Math.floor(Math.random() * 256)
  return bytes
}

module.exports = {
  constants: { RSA_NO_PADDING: 3 },
  createHash,
  createPublicKey,
  publicEncrypt,
  randomBytes,
}
