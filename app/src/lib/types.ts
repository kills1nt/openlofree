// Mirrors the Rust DTOs in src-tauri/src/dto.rs and flow2-core. Field names are camelCase there.
export type Mode = 'off' | 'steady' | 'breathing';

export interface KeyDef {
  row: number;
  col: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface ConnectInfo {
  modelId: string;
  label: string;
  verified: boolean;
  demo: boolean;
  mac: boolean;
  product: string;
  vendorId: number;
  productId: number;
  protocol: number;
  layers: number;
  rows: number;
  cols: number;
  keys: KeyDef[];
}

export interface Backlight {
  mode: Mode;
  /** 0 to 255, the raw VIA value. */
  brightness: number;
}

export type Layers = number[][];

export interface DeviceState {
  layers: Layers;
  backlight: Backlight;
}

export interface ProfileInfo {
  name: string;
  model: string;
  layers: number;
}

export interface CatalogItem {
  code: number;
  label: string;
}

export interface CatalogGroup {
  name: string;
  items: CatalogItem[];
}

export interface BackupInfo {
  path: string | null;
  exists: boolean;
}

export interface AppError {
  kind: string;
  message: string;
}

export interface Api {
  connect(demo: boolean): Promise<ConnectInfo>;
  disconnect(): Promise<void>;
  readState(): Promise<DeviceState>;
  setKey(layer: number, row: number, col: number, code: number): Promise<void>;
  applyKeymap(layers: Layers): Promise<number>;
  previewBacklight(backlight: Backlight): Promise<void>;
  applyBacklight(backlight: Backlight): Promise<void>;
  listProfiles(): Promise<ProfileInfo[]>;
  saveProfile(name: string, layers: Layers, backlight: Backlight): Promise<ProfileInfo[]>;
  applyProfile(name: string): Promise<number>;
  duplicateProfile(name: string, newName: string): Promise<ProfileInfo[]>;
  deleteProfile(name: string): Promise<ProfileInfo[]>;
  exportProfile(name: string): Promise<boolean>;
  importProfile(): Promise<ProfileInfo[] | null>;
  battery(): Promise<number | null>;
  catalog(): Promise<CatalogGroup[]>;
  legends(codes: number[]): Promise<string[]>;
  factoryBackupInfo(): Promise<BackupInfo>;
  onProfileApplied(handler: (name: string, changed: number) => void): Promise<() => void>;
  onTrayError(handler: (message: string) => void): Promise<() => void>;
}
