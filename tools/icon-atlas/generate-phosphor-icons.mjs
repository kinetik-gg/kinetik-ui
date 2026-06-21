import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { fileURLToPath } from "node:url";

const TOOL_ROOT = dirname(fileURLToPath(import.meta.url));

const ICON_SIZE = 32;
const PADDING = 1;
const CELL_SIZE = ICON_SIZE + PADDING * 2;
const COLUMNS = 7;
const ICONS = [
  [7001, "ICON_CURSOR", "cursor", "cursor"],
  [7002, "ICON_MOVE", "arrows-out-cardinal", "move"],
  [7003, "ICON_TRANSFORM", "bounding-box", "transform"],
  [7004, "ICON_ROTATE", "arrow-clockwise", "rotate"],
  [7005, "ICON_CUBE", "cube", "cube"],
  [7006, "ICON_PLAY", "play", "play"],
  [7007, "ICON_PAUSE", "pause", "pause"],
  [7008, "ICON_STOP", "stop", "stop"],
  [7009, "ICON_PLUS", "plus", "plus"],
  [7010, "ICON_SEARCH", "magnifying-glass", "search"],
  [7011, "ICON_ARCHIVE", "archive", "archive"],
  [7012, "ICON_FILE", "file", "file"],
  [7013, "ICON_IMAGE", "image", "image"],
  [7014, "ICON_GEAR", "gear", "gear"],
  [7015, "ICON_GRID", "grid-four", "grid"],
  [7016, "ICON_LAYERS", "stack", "layers"],
  [7017, "ICON_CODE", "code", "code"],
  [7018, "ICON_BOX", "package", "package"],
  [7019, "ICON_ROCKET", "rocket", "rocket"],
  [7020, "ICON_DOWNLOAD", "download", "download"],
  [7021, "ICON_DOTS", "dots-three", "more"],
  [7022, "ICON_CHEVRON", "caret-right", "chevron-right"],
  [7023, "ICON_CARET", "caret-down", "caret-down"],
  [7024, "ICON_RESET", "arrow-counter-clockwise", "reset"],
  [7025, "ICON_COMPONENT", "circles-four", "component"],
  [7026, "ICON_TOKENS", "swatches", "tokens"],
  [7027, "ICON_EYE", "eye", "visibility"],
  [7028, "ICON_CROSSHAIR", "crosshair", "crosshair"],
];

function parseArgs(argv) {
  const args = new Map();
  for (let i = 2; i < argv.length; i += 2) {
    if (!argv[i]?.startsWith("--")) throw new Error(`Expected --flag, got ${argv[i]}`);
    args.set(argv[i].slice(2), argv[i + 1]);
  }
  return args;
}

function rasterizeIcon({ sourceRoot, tmpRoot, icon }) {
  const [, symbol, sourceName] = icon;
  const svgPath = join(sourceRoot, "assets", "regular", `${sourceName}.svg`);
  let svg = readFileSync(svgPath, "utf8");
  svg = svg.replaceAll("currentColor", "white");
  const tempSvg = join(tmpRoot, `${symbol}.svg`);
  const tempRgba = join(tmpRoot, `${symbol}.rgba`);
  writeFileSync(tempSvg, svg);
  execFileSync("magick", ["-background", "none", tempSvg, "-resize", `${ICON_SIZE}x${ICON_SIZE}`, "-depth", "8", tempRgba], { stdio: "inherit" });
  const bytes = readFileSync(tempRgba);
  const expected = ICON_SIZE * ICON_SIZE * 4;
  if (bytes.length !== expected) throw new Error(`${sourceName} produced ${bytes.length} bytes, expected ${expected}`);
  return bytes;
}

function copyPixel(source, sourceWidth, sx, sy, dest, destWidth, dx, dy) {
  const sourceIndex = (sy * sourceWidth + sx) * 4;
  const destIndex = (dy * destWidth + dx) * 4;
  dest[destIndex] = source[sourceIndex];
  dest[destIndex + 1] = source[sourceIndex + 1];
  dest[destIndex + 2] = source[sourceIndex + 2];
  dest[destIndex + 3] = source[sourceIndex + 3];
}

function packAtlas(rasters) {
  const rows = Math.ceil(ICONS.length / COLUMNS);
  const width = COLUMNS * CELL_SIZE;
  const height = rows * CELL_SIZE;
  const atlas = Buffer.alloc(width * height * 4);
  for (let index = 0; index < rasters.length; index += 1) {
    const raster = rasters[index];
    const column = index % COLUMNS;
    const row = Math.floor(index / COLUMNS);
    const x0 = column * CELL_SIZE;
    const y0 = row * CELL_SIZE;
    for (let y = 0; y < CELL_SIZE; y += 1) {
      const sy = Math.max(0, Math.min(ICON_SIZE - 1, y - PADDING));
      for (let x = 0; x < CELL_SIZE; x += 1) {
        const sx = Math.max(0, Math.min(ICON_SIZE - 1, x - PADDING));
        copyPixel(raster, ICON_SIZE, sx, sy, atlas, width, x0 + x, y0 + y);
      }
    }
  }
  return { atlas, width, height, rows };
}

function writeManifest({ outputRoot, packageJson, width, height, rows }) {
  const manifest = {
    schemaVersion: 1,
    source: { package: packageJson.name, version: packageJson.version, repository: packageJson.repository, license: packageJson.license, weight: "regular", pathLayout: "assets/regular/<icon>.svg" },
    raster: { format: "rgba8", iconSize: ICON_SIZE, padding: PADDING, cellSize: CELL_SIZE, columns: COLUMNS, rows, atlasWidth: width, atlasHeight: height, colorModel: "white-alpha mask; runtime tint applies color" },
    icons: ICONS.map(([idRaw, symbol, sourceName, alias], index) => {
      const column = index % COLUMNS;
      const row = Math.floor(index / COLUMNS);
      return { idRaw, symbol, alias, sourceName, sourceSvg: `assets/regular/${sourceName}.svg`, sourceRect: { x: column * CELL_SIZE + PADDING, y: row * CELL_SIZE + PADDING, width: ICON_SIZE, height: ICON_SIZE } };
    }),
  };
  writeFileSync(join(outputRoot, "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
}

function writeRustMetadata({ outputRoot, width, height }) {
  const lines = [];
  lines.push("// @generated by tools/icon-atlas/generate-phosphor-icons.mjs");
  lines.push("");
  lines.push("use kinetik_ui::core::{ImageId, Rect};");
  lines.push("");
  lines.push("pub(super) struct PhosphorIconEntry {");
  lines.push("    pub image: ImageId,");
  lines.push("    pub symbol: &'static str,");
  lines.push("    pub source_name: &'static str,");
  lines.push("    pub source: Rect,");
  lines.push("}");
  lines.push("");
  lines.push("pub(super) const ICON_ATLAS: ImageId = ImageId::from_raw(7_000);");
  lines.push(`pub(super) const ICON_ATLAS_WIDTH: u32 = ${width};`);
  lines.push(`pub(super) const ICON_ATLAS_HEIGHT: u32 = ${height};`);
  lines.push(`pub(super) const ICON_SIZE: u32 = ${ICON_SIZE};`);
  lines.push(`pub(super) const ICON_ATLAS_PADDING: u32 = ${PADDING};`);
  lines.push(`pub(super) const ICON_ATLAS_CELL_SIZE: u32 = ${CELL_SIZE};`);
  lines.push(`pub(super) const ICON_ATLAS_COLUMNS: u32 = ${COLUMNS};`);
  lines.push(`pub(super) const ICON_ATLAS_ROWS: u32 = ${Math.ceil(ICONS.length / COLUMNS)};`);
  lines.push("pub(super) const ICON_ATLAS_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/icons/phosphor/atlas.rgba\"));");
  lines.push("");
  for (const [idRaw, symbol] of ICONS) lines.push(`pub(super) const ${symbol}: ImageId = ImageId::from_raw(${idRaw});`);
  lines.push("");
  lines.push("pub(super) const ICON_ENTRIES: &[PhosphorIconEntry] = &[");
  for (const [idRaw, symbol, sourceName] of ICONS) {
    const index = ICONS.findIndex((icon) => icon[0] === idRaw);
    const column = index % COLUMNS;
    const row = Math.floor(index / COLUMNS);
    const x = column * CELL_SIZE + PADDING;
    const y = row * CELL_SIZE + PADDING;
    lines.push("    PhosphorIconEntry {");
    lines.push(`        image: ${symbol},`);
    lines.push(`        symbol: \"${symbol}\",`);
    lines.push(`        source_name: \"${sourceName}\",`);
    lines.push(`        source: Rect::new(${x}.0, ${y}.0, ${ICON_SIZE}.0, ${ICON_SIZE}.0),`);
    lines.push("    },");
  }
  lines.push("];");
  writeFileSync(join(outputRoot, "phosphor_icons.rs"), `${lines.join("\n")}\n`);
}

const args = parseArgs(process.argv);
const sourceRoot = resolve(args.get("source") ?? join(TOOL_ROOT, "node_modules", "@phosphor-icons", "core"));
const outputRoot = resolve(args.get("output") ?? "apps/kinetik-ui-showcase/assets/icons/phosphor");
const packageJson = JSON.parse(readFileSync(join(sourceRoot, "package.json"), "utf8"));
mkdirSync(outputRoot, { recursive: true });
const tmpRoot = join(tmpdir(), `kinetik-phosphor-${Date.now()}`);
mkdirSync(tmpRoot, { recursive: true });
try {
  const rasters = ICONS.map((icon) => rasterizeIcon({ sourceRoot, tmpRoot, icon }));
  const { atlas, width, height, rows } = packAtlas(rasters);
  const atlasPath = join(outputRoot, "atlas.rgba");
  const atlasPngPath = join(outputRoot, "atlas.png");
  writeFileSync(atlasPath, atlas);
  execFileSync("magick", ["-size", `${width}x${height}`, "-depth", "8", `rgba:${atlasPath}`, atlasPngPath], { stdio: "inherit" });
  writeManifest({ outputRoot, packageJson, width, height, rows });
  writeRustMetadata({ outputRoot, width, height });
  console.log(`Generated ${ICONS.length} Phosphor icons into ${outputRoot}`);
} finally {
  rmSync(tmpRoot, { recursive: true, force: true });
}
