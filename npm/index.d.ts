export interface Migration {
  version: string;
  name: string;
}

export interface MigrationStatus {
  version: string;
  name: string;
  applied: boolean;
  appliedAt?: string;
}

export interface RunResult {
  success: boolean;
  applied: number;
  migrations: Migration[];
  error?: string;
}

export interface StatusResult {
  success: boolean;
  migrations: MigrationStatus[];
  error?: string;
}

export interface CreateResult {
  success: boolean;
  path: string;
  error?: string;
}

export interface RemoveResult {
  success: boolean;
  removed?: string;
  message?: string;
  error?: string;
}

export declare class Migrator {
  constructor(migrationsDir: string);
  run(): RunResult;
  status(): StatusResult;
  create(name: string, content: string): CreateResult;
  removePending(): RemoveResult;
}

export { Migrator as default };
