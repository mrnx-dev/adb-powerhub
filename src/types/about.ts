export type BuildEnvironment = 'production' | 'development';

export interface AppInfo {
  name: string;
  version: string;
  commit: string;
  environment: BuildEnvironment;
}

export interface DependencyStatus {
  name: string;
  path: string;
  version: string | null;
  available: boolean;
}

export interface DebugInfo {
  app: AppInfo;
  dependencies: DependencyStatus[];
  platform: string;
  arch: string;
}

export interface UpdateCheckResult {
  available: boolean;
  version: string | null;
  url: string | null;
  message: string;
}
