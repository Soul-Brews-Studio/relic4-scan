// Every word on the landing page, in one place. Plain strings; `backticks` mark code. app.js renders it; site/seo.mjs
// writes the same text into index.html (JSON-LD and the no-JavaScript page) and into llms.txt and llms-full.txt.
// Numbers here are measured (see PROOF_NOTE); nothing is invented.

export const SITE = {
  name: "ARRA Relic Scan",
  url: "https://relic.buildwithoracle.com/scan/",
  org: "Soul Brews Studio",
  tagline: "Every agent transcript, found and kept.",
  summary: "ARRA Relic Scan finds every AI-agent transcript on your machine (Claude Code, Codex, OMP, OMX, Gemini, Pi), indexes it into one Parquet file per month, and lets you browse it locally. Nothing leaves the machine.",
  version: "26.10.23-alpha.923",
};
const REL = "https://github.com/Soul-Brews-Studio/relic4-scan/releases";
export const DL = {
  mac: `${REL}/download/v${SITE.version}/ARRA-Relic-Scan_${SITE.version}_macos-arm64.dmg`,
  deb: `${REL}/download/v${SITE.version}/ARRA-Relic-Scan_${SITE.version}_linux-amd64.deb`,
  appimage: `${REL}/download/v${SITE.version}/ARRA-Relic-Scan_${SITE.version}_linux-amd64.AppImage`,
  notes: `${REL}/tag/v${SITE.version}`,
};

export const HERO = {
  cmd: "relic-scan run",
  lede: "Claude Code, Codex, OMP, OMX, Gemini and Pi leave thousands of `.jsonl` files across homes, mirrors and archives.",
  claim: "ARRA Relic Scan finds every one, indexes it into one Parquet file per month, and lets you browse it on your own machine.",
  close: "Nothing leaves it.",
  note: "Private beta · the downloads need access to Soul-Brews-Studio on GitHub · Apple silicon, Linux x86_64",
};

export const PROOF = [
  ["58 s", "to index 86,904 transcripts (75.6 GB) from cold"],
  ["≈15 s", "to bring that index up to date, reading only new bytes"],
  ["5.5 M", "rows: every message, thinking block, tool call and result"],
  ["0", "network calls from the app or its page"],
];
export const PROOF_NOTE = "Measured on one Apple-silicon Mac, October 2026, over its own agents' history. Your numbers follow your disk.";

export const TERMINAL = {
  h: "Start in the terminal",
  p: "`relic-scan tui` reads the index: overview, repositories, sessions and every `.jsonl`.",
  keys: "`/` filters · `Enter` opens · `⌫` goes back · no background of its own, so it reads on dark and light terminals",
};

export const STEPS = [
  { id: "scan", dot: "Smart Scan", shot: "scan", h: "The whole index at a glance", p: "Smart Scan shows every row by kind, the sessions and files behind them, and how far back your history goes.",
    li: ["user text, AI text, thinking, tool calls, results and errors", "Check changes and Index now, one click each", "the output folder under every title: Open folder, Export list"] },
  { id: "check", dot: "Check", shot: "scans", h: "Check before you index", p: "Check changes walks your folders and says what an index would read. It writes nothing.",
    li: ["new, grown, rewritten and gone files, with the bytes to read", "every index and every check kept in a scan log you can open", "Stop ends a run during the walk or the parse, with nothing written"] },
  { id: "contents", dot: "Contents", shot: "scan-contents", h: "See what your agents wrote", p: "Pick a kind of row and see where it came from.",
    li: ["main sessions, subagents and workflow agents", "own homes, mirrors, archives and rescued copies", "agent by agent, month by month, stacked by kind"] },
  { id: "map", dot: "Session map", shot: "map", h: "Every session as a star", p: "Rings are the months a session started; sectors are repositories. Subagents orbit the session that started them.",
    li: ["a query language: `repo:` `month:` `tier:` `errors>=20`", "click a star for its rows, its copies and `claude --resume`", "⌘K finds any session, repo, home, file or scan"] },
  { id: "tree", dot: "Tree", shot: "tree", h: "Home, repository, session", p: "The same index as a tree, newest first, with rows by kind on every line.",
    li: ["open a bank and see only that bank, with the way back", "export what you see as CSV or JSON, with the time and the query"] },
  { id: "disk", dot: "Disk", shot: "disk", h: "Where the gigabytes are", p: "Every `.jsonl` by folder and size, as rings, one colour per branch. Click to go in; open any folder in Finder.",
    li: ["indexed and not-indexed bytes for every folder", "folders the scan could not read, listed in Health"] },
];

export const SHOTS = {
  scan: "ARRA Relic Scan's Smart Scan page: a donut of rows by kind (user text, AI text, tool calls, tool results, thinking, tool errors) and the totals: rows, sessions, files indexed, newest row and span.",
  scans: "The scan log: one row per index or check with new, grown, rewritten and gone files, bytes read and time.",
  "scan-contents": "The contents view: bars for the kind of .jsonl, where the file lives and the agent, and a monthly bar chart stacked by kind of row.",
  map: "The session map: hundreds of dots on rings by month and sectors by repository, with a panel of totals and rows by type.",
  tree: "The tree: a home folder opened to its repositories, each with session and row counts and a bar of rows by kind.",
  disk: "The disk view: rings of folders sized by bytes, one colour per branch, with a legend of the folder's children and their sizes.",
};
export const TUI_ALT = "A recording of relic-scan tui: the overview, then repositories, one opened to its sessions, sessions filtered by billing, and back.";

export const LOCAL = {
  h: "It stays on your machine.",
  facts: [
    ["127.0.0.1", "The page is served on this machine only, and makes no network calls."],
    ["on a click", "Nothing is written until you press Index now. Checks only read."],
    ["~/.relic-scan", "Your index is plain Parquet: one file per month, plus sessions, files and every .jsonl."],
    ["four ways in", "A desktop app, the page in any browser, a terminal UI, and the `relic-scan` command."],
  ],
  run: ["run: scan 9.4s · new 0 · grown 1 · rewritten 5 · unchanged 86,900 · gone 0", "merge: 1 months in 4.4s · total 4.5s",
    "meta: 15 months (1 re-aggregated) · 86906 files · 45095 sessions", "map: 45095 sessions · 540 repos · 73 homes", "run: done in 15.4s"],
  card: ["45,095 sessions · 5,549,427 rows · 86,906 files · 0.0 GB read", "new 0 · grown 1 · rewritten 5 · unchanged 86,900",
    "scan 9.4s · parse+merge 4.7s · meta 1.1s · map 0.2s · total 15.4s"],
  caption: "A real incremental run on the same Mac: six files had changed, so it read only those.",
};

// questions people (and answer engines) ask, each answered in its first sentence
export const FAQ = [
  ["What is ARRA Relic Scan?", "A desktop app and command-line tool that finds every AI-agent transcript on your machine, indexes it into one Parquet file per month, and lets you browse it locally: a dashboard, a session map, a tree, a disk view and a terminal UI."],
  ["Which agents does it read?", "Claude Code, Codex, OMP, OMX, Gemini, Pi and thClaws: any .claude, .codex, .omp, .omx, .gemini, .pi or .thclaws home, plus backups, mirrors and archived copies of them."],
  ["Does it upload my transcripts anywhere?", "No. The app serves its page on 127.0.0.1 only, the page makes no network calls, and the index is written to ~/.relic-scan on your own disk."],
  ["Does it change my transcript files?", "No. It only reads them; everything it writes goes to its own folder (~/.relic-scan), and Check changes writes nothing at all."],
  ["How fast is it?", "On one Apple-silicon Mac it indexed 86,904 transcripts (75.6 GB) in 58 seconds from cold, and brings that index up to date in about 15 seconds by reading only new bytes."],
  ["What format is the index?", "Plain Parquet: one file per month of rows (every message, thinking block, tool call and result), plus sessions, files and every .jsonl, readable by DuckDB, Polars or pandas."],
  ["Which platforms does it run on?", "macOS on Apple silicon (a .dmg), and Linux x86_64 (.deb and .AppImage); the relic-scan command builds on macOS and Linux with Rust."],
  ["Is it free and open source?", "It is a private beta: the downloads sit in a private GitHub repository and need access to the Soul-Brews-Studio organisation."],
];

export const GET = {
  p: `Version ${SITE.version}, a private beta. The files sit in a private GitHub repository: sign in with an account that has access to Soul-Brews-Studio.`,
  rows: [
    ["macOS", "Apple silicon · drag into Applications", ".dmg · 6.6 MB", DL.mac],
    ["Linux", "x86_64 · Debian, Ubuntu", ".deb · 7.5 MB", DL.deb],
    ["Linux", "x86_64 · any distribution", ".AppImage · 85 MB", DL.appimage],
  ],
  cli: "gh api repos/Soul-Brews-Studio/relic4-scan/contents/install.sh \\\n  -H 'Accept: application/vnd.github.raw' | sh",
};
export const FOOTER = { say: "Your agents keep everything they did. Now you can find it.", note: "Every capture on this page comes from a demo index of made-up repositories." };
