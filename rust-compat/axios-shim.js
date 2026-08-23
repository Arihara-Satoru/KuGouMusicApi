function encode(value) {
  return JSON.stringify(value, (_key, item) => {
    if (Buffer.isBuffer(item)) return { __kugou_buffer__: item.toString('base64') }
    if (item && item.type === 'Buffer' && Array.isArray(item.data)) {
      return { __kugou_buffer__: Buffer.from(item.data).toString('base64') }
    }
    return item
  })
}

function decode(value) {
  return JSON.parse(value, (_key, item) =>
    item && typeof item === 'object' && typeof item.__kugou_buffer__ === 'string'
      ? Buffer.from(item.__kugou_buffer__, 'base64')
      : item
  )
}

async function axios(options = {}) {
  const response = decode(await globalThis.__rust_http(encode(options)))
  if (response.__error__) {
    const error = new Error(response.message || 'request failed')
    if (response.response) error.response = response.response
    throw error
  }
  return response
}

axios.get = (url, options = {}) => axios({ ...options, method: 'GET', url })
axios.delete = (url, options = {}) => axios({ ...options, method: 'DELETE', url })
axios.post = (url, data, options = {}) => axios({ ...options, method: 'POST', url, data })
axios.put = (url, data, options = {}) => axios({ ...options, method: 'PUT', url, data })
axios.request = axios
axios.default = axios

module.exports = axios
