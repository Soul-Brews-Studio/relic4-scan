#!/usr/bin/env python3
"""A demo home of made-up repositories for the guide book: Claude Code, Codex and omp transcripts plus two ψ vaults.
Nothing here is real: the repos, people, prompts and answers are invented. Deterministic (fixed seed).
  python3 -I make-demo-home.py <dir>      # writes <dir>/.claude, <dir>/.codex, <dir>/.omp, <dir>/Code/…/ψ
"""
import json, os, random, sys, uuid, datetime as dt

OUT = os.path.abspath(sys.argv[1])
# the real relic-scan walks ~ and /opt/Code: a demo home there is indexed as if it were real (it was, on 2026-10-26, and had to be cut out)
for bad in (os.path.expanduser("~"), "/opt/Code"):
    if OUT == bad or OUT.startswith(bad + os.sep):
        sys.exit(f"refusing: {OUT} is under {bad}, which relic-scan indexes. Use a folder outside it, e.g. /tmp/relic-demo.")
R = random.Random(2610)
REPOS = [  # (dir name, branch, what it is)
    ("starfish-api", "main", "a REST API for a fish-tank shop"),
    ("lantern-app", "feat/offline", "a mobile app that reads lantern-festival maps"),
    ("kite-docs", "docs/tour", "a documentation site for a kite club"),
    ("moss-cli", "main", "a command line tool that waters plants"),
    ("harbor-bot", "fix/retry", "a chat bot for a small harbour office"),
    ("tea-cart", "main", "a checkout page for a tea cart"),
]
ASKS = [
    ("add a retry with backoff to the order sync", "I'll wrap the call in a retry loop with exponential backoff and a cap of five tries."),
    ("why does the offline map flicker on first load", "The tile layer mounts before the cache is read; reading the cache first removes the flicker."),
    ("write the tour page for new members", "I drafted the page with three steps: join, borrow a kite, fly it. Check the tone."),
    ("the watering schedule skips sundays, find out why", "The cron expression uses day 7 for Sunday, but this scheduler counts from 0. Changing it to 0 fixes it."),
    ("make the harbour bot answer in two languages", "I added a language flag per chat and a small table of replies in both languages."),
    ("the cart total is off by one satang", "The total is rounded per line, then summed. Summing first and rounding once gives the right number."),
    ("rename the config key tank_ids to tanks", "Renamed in the loader, the tests and the docs; the old key still loads with a warning."),
    ("add a test for the empty basket", "Added two tests: an empty basket returns zero, and a basket with one free item still shows the line."),
    ("summarise yesterday's errors from the log", "Three kinds: two timeouts to the payment sandbox, one bad date, and a burst of 404s from a crawler."),
    ("can you check the release notes for typos", "Found four: 'recieve', 'seperate', a doubled 'the', and a wrong version number in the heading."),
]
TOOLS = [("Bash", {"command": "rg -n retry src | head"}, "src/sync.py:41:    for attempt in range(3):\nsrc/sync.py:58:def retry(fn):"),
         ("Read", {"file_path": "src/sync.py"}, "def sync(orders):\n    for o in orders:\n        push(o)\n"),
         ("Edit", {"file_path": "src/sync.py", "old_string": "range(3)", "new_string": "range(5)"}, "The file src/sync.py has been updated."),
         ("Bash", {"command": "pytest -q"}, "....F.\n1 failed, 5 passed in 0.42s")]
THINK = ["The user wants a small change. Check the call site first, then the tests.", "Two ways to do this; the smaller one is enough.",
         "Run the tests before and after so the result is a fact, not a guess."]

def iso(t): return t.strftime("%Y-%m-%dT%H:%M:%S.") + f"{t.microsecond//1000:03d}Z"
def write(path, lines):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        for l in lines: f.write(json.dumps(l, ensure_ascii=False) + "\n")

def claude_session(home, repo, branch, start, n_turns, model, title=None, secret=False):
    sid = str(uuid.UUID(int=R.getrandbits(128), version=4))
    cwd = f"/opt/Code/github.com/laris-co/{repo}"
    proj = "-opt-Code-github-com-laris-co-" + repo
    t = start; lines = []; common = dict(sessionId=sid, cwd=cwd, gitBranch=branch, version="2.1.40")
    for i in range(n_turns):
        ask, ans = R.choice(ASKS)
        if secret and i == 0: ask += " (the key is sk-demo-0000000000000000000000000000, do not print it)"
        t += dt.timedelta(seconds=R.randint(20, 200))
        lines.append(dict(type="user", timestamp=iso(t), message=dict(role="user", content=ask), **common))
        mid = f"msg_{sid[:8]}_{i}"
        t += dt.timedelta(seconds=R.randint(3, 15))
        usage = dict(input_tokens=R.randint(3, 40), output_tokens=R.randint(180, 2400), cache_read_input_tokens=R.randint(8000, 90000), cache_creation_input_tokens=R.randint(0, 6000))
        lines.append(dict(type="assistant", timestamp=iso(t), message=dict(id=mid, role="assistant", model=model, usage=usage,
            content=[dict(type="thinking", thinking=R.choice(THINK) if R.random() < .5 else "", signature="demo")]), **common))
        for k in range(R.randint(0, 3)):
            name, inp, res = R.choice(TOOLS); tid = f"toolu_{sid[:6]}_{i}_{k}"
            t += dt.timedelta(seconds=R.randint(2, 20))
            lines.append(dict(type="assistant", timestamp=iso(t), message=dict(id=mid, role="assistant", model=model, usage=usage,
                content=[dict(type="tool_use", id=tid, name=name, input=inp)]), **common))
            t += dt.timedelta(seconds=R.randint(1, 8))
            lines.append(dict(type="user", timestamp=iso(t), message=dict(role="user", content=[dict(type="tool_result", tool_use_id=tid, content=res, is_error=res.startswith("....F"))]), **common))
        t += dt.timedelta(seconds=R.randint(2, 10))
        lines.append(dict(type="assistant", timestamp=iso(t), message=dict(id=mid, role="assistant", model=model, usage=usage, content=[dict(type="text", text=ans)]), **common))
    if title: lines.append(dict(type="custom-title", customTitle=title, sessionId=sid))
    write(f"{home}/.claude/projects/{proj}/{sid}.jsonl", lines)
    return sid, t

def codex_session(home, repo, start, n):
    sid = str(uuid.UUID(int=R.getrandbits(128), version=4)); t = start
    day = start.strftime("%Y/%m/%d")
    lines = [dict(type="session_meta", timestamp=iso(t), payload=dict(id=sid, cwd=f"/Users/demo/Code/{repo}")),
             dict(type="turn_context", timestamp=iso(t), payload=dict(model="gpt-6-astra", cwd=f"/Users/demo/Code/{repo}"))]
    for i in range(n):
        ask, ans = R.choice(ASKS)
        t += dt.timedelta(seconds=R.randint(20, 120))
        lines.append(dict(type="response_item", timestamp=iso(t), payload=dict(type="message", role="user", content=[dict(type="input_text", text=ask)])))
        t += dt.timedelta(seconds=R.randint(3, 30))
        lines.append(dict(type="response_item", timestamp=iso(t), payload=dict(type="message", role="assistant", content=[dict(type="output_text", text=ans)])))
    write(f"{home}/.codex/sessions/{day}/rollout-{start.strftime('%Y-%m-%dT%H-%M-%S')}-{sid}.jsonl", lines)

def omp_session(home, repo, start, n):
    sid = str(uuid.UUID(int=R.getrandbits(128), version=4)); t = start; lines = []
    for i in range(n):
        ask, ans = R.choice(ASKS)
        t += dt.timedelta(seconds=R.randint(20, 120))
        lines.append(dict(type="message", id=f"u{i}", timestamp=iso(t), message=dict(role="user", content=[dict(type="text", text=ask)])))
        t += dt.timedelta(seconds=R.randint(3, 30))
        lines.append(dict(type="message", id=f"a{i}", timestamp=iso(t), message=dict(role="assistant", model="glm-5.3", responseId=f"r_{sid[:6]}_{i}",
            usage=dict(input=R.randint(100, 900), output=R.randint(100, 900), cacheRead=R.randint(0, 5000), cacheWrite=0), content=[dict(type="text", text=ans)])))
    write(f"{home}/.omp/agent/sessions/-Users-demo-Code-{repo}/{start.strftime('%Y-%m-%dT%H-%M-%S-000Z')}_{sid}.jsonl", lines)

def vault(root, name, notes):
    base = f"{root}/Code/{name}"
    os.makedirs(f"{base}/.git", exist_ok=True)
    for sec, items in notes.items():
        for fn, title in items:
            p = f"{base}/ψ/{sec}/{fn}"; os.makedirs(os.path.dirname(p), exist_ok=True)
            open(p, "w").write(f"# {title}\n\nA note written for the demo. Nothing here is real.\n")

def main():
    months = [dt.datetime(2026, m, 1, 3, 0) for m in (7, 8, 9, 10)]
    models = ["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5"]
    n = 0
    for mi, m0 in enumerate(months):
        for d in range(1, 28 if m0.month < 10 else 25):
            repo, br, _ = R.choice(REPOS)
            start = m0 + dt.timedelta(days=d, hours=R.randint(0, 10), minutes=R.randint(0, 59))
            sid, _ = claude_session(OUT, repo, br, start, R.randint(2, 7), R.choice(models),
                                    title=(f"{repo}: session {n}" if n % 4 == 0 else None), secret=(n == 7)); n += 1
            if d % 5 == 1: codex_session(OUT, R.choice(REPOS)[0], start + dt.timedelta(hours=2), R.randint(2, 5))
            if d % 7 == 3: omp_session(OUT, R.choice(REPOS)[0], start + dt.timedelta(hours=4), R.randint(2, 4))
    vault(OUT, "kite-docs", {"memory/learnings": [("2026-08-02_tone.md", "Write the tour in short steps"), ("2026-09-11_images.md", "One kite photo per step")],
                             "inbox": [("todo.md", "Todo for the tour page")]})
    vault(OUT, "moss-cli", {"memory/learnings": [("2026-07-20_cron.md", "Cron counts Sunday as 0 here")], "retrospectives": [("2026-10-01_week.md", "Week of the watering fix")]})
    print(f"demo home: {n} Claude sessions in {OUT}")
main()
