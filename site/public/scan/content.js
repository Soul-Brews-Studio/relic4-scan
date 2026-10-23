// Every word on the landing page, in one place. Plain strings; `backticks` mark code. app.js renders it; site/seo.mjs
// writes the same text into index.html (JSON-LD and the no-JavaScript page) and into llms.txt and llms-full.txt.
// Numbers here are measured (see PROOF_NOTE); nothing is invented.

export const SITE = {
  name: "ARRA Relic Scan",
  url: "https://relic.buildwithoracle.com/scan/",
  org: "Soul Brews Studio",
  tagline: "Every agent transcript, found and kept.",
  summary: "ARRA Relic Scan finds every AI-agent transcript on your machine (Claude Code, Codex, OMP, OMX, Gemini, Pi), indexes it into one Parquet file per month, and lets you browse it locally. Nothing leaves the machine.",
  version: "26.10.23-alpha.2312",
  repo: "https://github.com/Soul-Brews-Studio/relic4-scan",
  license: "GNU AGPL v3",
};
const REL = "https://github.com/Soul-Brews-Studio/relic4-scan/releases";
export const DL = {
  mac: `${REL}/download/v${SITE.version}/ARRA-Relic-Scan_${SITE.version}_macos-arm64.dmg`,
  intel: `${REL}/download/v${SITE.version}/ARRA-Relic-Scan_${SITE.version}_macos-x64.dmg`,
  deb: `${REL}/download/v${SITE.version}/ARRA-Relic-Scan_${SITE.version}_linux-amd64.deb`,
  appimage: `${REL}/download/v${SITE.version}/ARRA-Relic-Scan_${SITE.version}_linux-amd64.AppImage`,
  notes: `${REL}/tag/v${SITE.version}`,
};

export const HERO = {
  cmd: "relic-scan run",
  lede: "Claude Code, Codex, OMP, OMX, Gemini and Pi leave thousands of `.jsonl` files across homes, mirrors and archives.",
  claim: "ARRA Relic Scan finds every one, indexes it into one Parquet file per month, and lets you browse it on your own machine.",
  close: "Nothing leaves it.",
  note: "Free and open source (AGPL-3.0) · Apple silicon and Intel Macs, Linux x86_64",
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
  { id: "vaults", dot: "ψ Vaults", shot: "vaults", h: "Every oracle's memory, too", p: "The same walk finds every `ψ` vault: an oracle's learnings, retrospectives, inbox and writing. Open one and its notes are grouped by folder, newest first.",
    li: ["each vault once, however many repos link to it", "git worktree copies kept apart, cloned repos inside skipped", "written as plain JSON: `meta/vaults.json`"] },
  { id: "read", dot: "Reader", shot: "reader", h: "Read a note where it lives", p: "Click a note and it opens rendered: front matter, headings, lists, code and tables. Nothing is copied, nothing leaves the machine.",
    li: ["filter a vault's notes by words in the title or path", "Show in Finder from the reader", "export a vault's notes as CSV"] },
  { id: "export", dot: "Export", shot: "export", h: "Take the metadata with you", p: "Every table the index holds, as CSV or JSON, in one place. The rows stay in one Parquet file per month, each one click from Finder.",
    li: ["sessions, files, homes, scans and vaults", "saved to ~/Downloads with the time it was exported", "click a month to show its file in Finder"] },
];

export const SHOTS = {
  scan: "ARRA Relic Scan's Smart Scan page: a donut of rows by kind (user text, AI text, tool calls, tool results, thinking, tool errors) and the totals: rows, sessions, files indexed, newest row and span.",
  scans: "The scan log: one row per index or check with new, grown, rewritten and gone files, bytes read and time.",
  "scan-contents": "The contents view: bars for the kind of .jsonl, where the file lives and the agent, and a monthly bar chart stacked by kind of row.",
  map: "The session map: hundreds of dots on rings by month and sectors by repository, with a panel of totals and rows by type.",
  tree: "The tree: a home folder opened to its repositories, each with session and row counts and a bar of rows by kind.",
  disk: "The disk view: rings of folders sized by bytes, one colour per branch, with a legend of the folder's children and their sizes.",
  vaults: "The ψ Vaults view: two oracle vaults with their notes counted by section; one is open, its notes grouped by folder (memory, inbox, writing) beside a list of learnings and retrospectives.",
  reader: "A learning open in the reader: its front matter, heading, paragraphs, a list and a shell command, rendered from the note's markdown.",
  export: "The Export view: sessions, files, homes, scans and vaults with their line counts and CSV and JSON buttons, then the month files with their sizes.",
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
  ["Does it read the same transcript twice?", "No. A ledger (~/.relic-scan/_ledger.tsv, appended, never rewritten) keeps a hash chain of every file it has read. A copy at a new path or from another machine is recognised and not read; a file that grew is read only from where the known version ends; a file whose past changed is flagged. On one Mac, moving a mirror of 852 transcripts cost 0.00 GB of reading."],
  ["What format is the index?", "Plain Parquet: one file per month of rows (every message, thinking block, tool call and result), plus sessions, files and every .jsonl, readable by DuckDB, Polars or pandas."],
  ["Which platforms does it run on?", "macOS on Apple silicon and on Intel (a .dmg each), and Linux x86_64 (.deb and .AppImage); the relic-scan command builds on macOS and Linux with Rust."],
  ["Can an AI agent install and use it?", "Yes. The repository's AGENTS.md (also at relic.buildwithoracle.com/scan/agents.md) tells a coding agent how to install, run, query and build it; the agent should ask before the first index, because it reads every transcript on the machine."],
  ["How does an agent query the index?", "With DuckDB, Polars or pandas over ~/.relic-scan, starting with a count of every row by what it is. Month files (YYYY-MM.parquet) hold every message and tool call: messages and thinking have a role (user, assistant, thinking), tool rows a kind (call, result); a thinking row with empty text is a block whose text was not saved. meta/sessions.parquet has one line per session; meta/vaults.json lists every ψ vault."],
  ["Is it free and open source?", "Yes. The source is on GitHub (github.com/Soul-Brews-Studio/relic4-scan) under the GNU AGPL v3, and every download is free."],
];

export const GET = {
  p: `Version ${SITE.version}, free and open source under the GNU AGPL v3. The source, the release notes and every file are on GitHub.`,
  rows: [
    ["macOS", "Apple silicon · drag into Applications", ".dmg · 6.8 MB", DL.mac],
    ["macOS", "Intel · drag into Applications", ".dmg · 7.1 MB", DL.intel],
    ["Linux", "x86_64 · Debian, Ubuntu", ".deb · 7.6 MB", DL.deb],
    ["Linux", "x86_64 · any distribution", ".AppImage · 85 MB", DL.appimage],
  ],
  cli: "curl -fsSL https://raw.githubusercontent.com/Soul-Brews-Studio/relic4-scan/main/install.sh | sh",
};
// build it from source: a justfile in the repository does the work
export const BUILD = {
  h: "Build it yourself",
  p: "The source is a Rust workspace with a `justfile`, on GitHub. You need Rust and `just`.",
  steps: [
    ["Get the source", "git clone https://github.com/Soul-Brews-Studio/relic4-scan && cd relic4-scan"],
    ["Get the tools", "curl https://sh.rustup.rs -sSf | sh   # Rust\nbrew install just                     # or: cargo install just"],
    ["Build the command", "just build     # target/release/relic-scan\njust install   # copies it into ~/.local/bin"],
    ["Build the desktop app", "just app         # .app and .dmg for this Mac\njust app-intel   # the same for Intel Macs, from Apple silicon\njust linux       # .deb and .AppImage, in Docker"],
  ],
  more: "`just` lists every recipe; `just test` runs the tests; `just up` indexes this machine and serves the page on 127.0.0.1.",
};

// for AI agents: a prompt a person can hand to their agent, and the queries an agent runs; site/seo.mjs publishes the
// repository's AGENTS.md as /scan/agents.md and points llms.txt at it
export const AGENT = {
  h: "For AI agents",
  p: "Hand this to Claude Code, Codex or any coding agent. It installs relic-scan, asks before it reads anything, and answers from the index on your machine.",
  prompt: "Install ARRA Relic Scan on this machine and tell me what my AI agents have been working on. Follow https://relic.buildwithoracle.com/scan/agents.md: install the relic-scan command, ask me before the first index, then run it and answer my questions from ~/.relic-scan with DuckDB. Keep everything on this machine.",
  queries: [
    ["everything, by what it is", "duckdb -c \"select coalesce(nullif(kind, ''), nullif(role, ''), '(none)') as what, count(*) as n from '*.parquet' group by 1 order by n desc\""],
    ["sessions per repository", "duckdb -c \"select cat as repo, count(*) as sessions from 'meta/sessions.parquet' group by 1 order by sessions desc limit 10\""],
    ["what I asked about something", "duckdb -c \"select ts_th, left(text, 120) as text from '*.parquet' where role = 'user' and text ilike '%parquet%' order by ts desc limit 20\""],
    ["the tools used most", "duckdb -c \"select name, count(*) as calls from '*.parquet' where kind = 'call' group by 1 order by calls desc limit 10\""],
    ["vaults with the most notes", "jq -r '.vaults[] | select(.kind != \"worktree\") | [.notes, .name] | @tsv' meta/vaults.json | head"],
  ],
  links: [["agents.md", "/scan/agents.md"], ["llms.txt", "/llms.txt"], ["AGENTS.md on GitHub", "https://github.com/Soul-Brews-Studio/relic4-scan/blob/main/AGENTS.md"]],
};

export const FOOTER = { license: "Open source under the GNU AGPL v3", say: "Your agents keep everything they did. Now you can find it.", note: "Every capture on this page comes from a demo index of made-up repositories." };
