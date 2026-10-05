const assert = require('node:assert/strict');
const { execFileSync } = require('node:child_process');

if (!process.argv[2]) {
  for (const runtime of ['source', 'bundled']) {
    for (const platform of ['default', 'lite']) {
      execFileSync(process.execPath, [__filename, platform, runtime], { stdio: 'inherit' });
    }
  }
  process.exit(0);
}

process.env.platform = process.argv[2];
const config = require('../util/config.json');
const expectedAppid = process.env.platform === 'lite' ? config.liteAppid : config.appid;
const expectedClientver = process.env.platform === 'lite' ? config.liteClientver : config.clientver;
for (const [name, route] of [['user_preference', 'get_user_conf'], ['user_preference_update', 'update_user_conf']]) {
  const root = process.argv[3] === 'bundled' ? '../bin/api_js' : '..';
  const result = require(`${root}/module/${name}`)({ cookie: { userid: 1, token: 'test-token' }, mode: '1' }, request => request);
  assert.equal(result.params.appid, expectedAppid);
  assert.equal(result.params.clientver, expectedClientver);
  assert.equal(result.url, `/userpreferservice/v1/${route}`);
  assert.ok(result.headers['User-Agent'].includes(`-${expectedClientver}-`));
  assert.equal(result.data.userid, 1);
  if (name.endsWith('_update')) assert.equal(result.data.data.mode, '1');
}
console.log(`user preference check passed (${process.argv[3]}, ${process.env.platform})`);
