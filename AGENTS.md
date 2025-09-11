# Repository Guidelines

## Project Structure & Module Organization
- `apps/web`: Vite + React app (served at `nx serve web`).
- `apps/web-e2e`: Playwright end‑to‑end tests for `web`.
- `packages/`: Reusable TS libraries — `drawnix`, `react-board`, `react-text`.
- `src-tauri/`: Tauri desktop wrapper (Rust + config).
- `dds_subscriber/`, `test_publisher/`: Rust crates for DDS testing.
- `scripts/`: Release/publish utilities.

## Build, Test, and Development Commands
- Install: `npm install`
- Dev (web): `npm run start` → runs `nx serve web`.
- Build all libs/apps: `npm run build` (Nx run-many).
- Unit tests: `npm test` (Jest via Nx); per project: `nx test packages/drawnix`.
- E2E: `nx e2e web-e2e` (Playwright).
- Desktop dev: `npm run tauri:dev` (builds/serves web then launches Tauri).
- DDS quick test: `bash ./test_dds.sh` (compiles crates and shows run steps).

## Coding Style & Naming Conventions
- Language: TypeScript/React (Vite) + Rust (Tauri/DDS).
- Formatting: Prettier (`singleQuote: true`), 2‑space indent. Run your editor’s Prettier on save.
- Linting: ESLint via Nx; respect module boundaries.
- Filenames: lower‑kebab‑case for files; export components in `PascalCase`.
- Paths: use TS aliases from `tsconfig.base.json` (e.g., `@drawnix/drawnix`).

## Testing Guidelines
- Unit tests colocated as `*.spec.ts(x)` (see `packages/*`).
- Run all: `npm test`; single project: `nx test <project>`; with coverage: `nx test <project> --coverage`.
- E2E specs live under `apps/web-e2e/src`; prefer realistic user flows.

## Commit & Pull Request Guidelines
- Commit style: Conventional Commits (e.g., `feat: ...`, `fix: ...`, `chore: ...`). Release script uses `chore(release): publish <version>` and tags `v<version>`.
- PRs: include a clear summary, linked issues, and screenshots/GIFs for UI changes. Note any breaking changes.

## Security & Configuration Tips
- Do not commit secrets. Review `src-tauri/tauri.conf.json` before packaging.
- Rust toolchain required for Tauri/DDS (`cargo build --release`). Ensure external path deps (e.g., `zrdds-safe`) are available when building desktop.

## Agent-Specific Instructions
- Keep changes minimal and consistent with Nx structure; don’t rename projects/targets.
- Prefer `nx` targets over raw tool invocations; avoid networked commands unless requested.
