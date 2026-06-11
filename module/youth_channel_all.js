module.exports = (params, useAxios) => {
  return useAxios({
    url: '/youth/v2/channel/channel_all_list',
    encryptType: 'android',
    method: 'get',
    // 文档只要求 page/pagesize；去掉额外的 type 参数，避免网关把请求判成非法参数。
    params: { page: params.page || 1, pagesize: params.pagesize || 30 },
    cookie: params?.cookie,
  });
};
