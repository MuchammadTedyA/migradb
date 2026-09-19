const { existsSync, readFileSync } = require('fs');
const { join } = require('path');

const pkg = require('./package.json');
const napi = pkg.napi;

function loadNativeBinding() {
  const platform = process.platform;
  const arch = process.arch;

  const platformMap = {
    'win32': 'win32',
    'linux': 'linux',
    'darwin': 'darwin',
    'freebsd': 'freebsd',
  };

  const archMap = {
    'x64': 'x64',
    'ia32': 'ia32',
    'arm64': 'arm64',
    'arm': 'arm',
  };

  const osPart = platformMap[platform];
  const archPart = archMap[arch];

  if (!osPart || !archPart) {
    throw new Error(`Unsupported platform: ${platform} ${arch}`);
  }

  const triple = `${archPart}-${osPart}`;
  const nativeDir = join(__dirname, 'native');

  const possiblePaths = [
    join(__dirname, 'migradb.node'),
    join(__dirname, 'index.node'),
    join(__dirname, `migradb.${triple}.node`),
    join(nativeDir, `migradb.${triple}.node`),
    join(nativeDir, `migration_engine_napi.${triple}.node`),
  ];

  for (const p of possiblePaths) {
    if (existsSync(p)) {
      return require(p);
    }
  }

  throw new Error(
    `Native binding not found for ${triple}. ` +
    `Please run 'npm run build' to compile the native module.`
  );
}

const native = loadNativeBinding();

module.exports = {
  Migrator: native.Migrator,
  Migration: native.Migration,
  MigrationStatus: native.MigrationStatus,
  RunResult: native.RunResult,
  StatusResult: native.StatusResult,
  CreateResult: native.CreateResult,
  RemoveResult: native.RemoveResult,
};
