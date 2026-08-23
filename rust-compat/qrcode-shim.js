module.exports = {
  toDataURL(text) {
    return globalThis.__rust_qrcode(String(text))
  },
}
