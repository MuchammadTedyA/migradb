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

  const possibleNames = [
    `migradb.${triple}.node`,
    `migration_engine_napi.${triple}.node`,
  ];

  for (const name of possibleNames) {
    const path = join(nativeDir, name);
    if (existsSync(path)) {
      return require(path);
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
