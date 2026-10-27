import json, sqlite3, urllib.request, collections, statistics

DB = "/Users/suze/Library/Application Support/com.chenal.studio/chenal.db"
UA = {"User-Agent": "Chenal/0.1.0 (macOS; research)"}


def fetch(url):
    req = urllib.request.Request(url, headers=UA)
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)


con = sqlite3.connect(DB)
con.row_factory = sqlite3.Row
games = con.execute(
    "select g.id, g.white_username, g.black_username, g.result, g.time_class, g.played_at,"
    " a.white_accuracy, a.black_accuracy, a.white_acpl, a.black_acpl, a.depth,"
    " (select count(*) from move_evaluations m where m.game_id=g.id) plies"
    " from games g join game_analyses a on a.game_id=g.id order by g.played_at"
).fetchall()

# group needed archives
months = collections.defaultdict(set)
for g in games:
    import datetime
    d = datetime.datetime.utcfromtimestamp(g["played_at"])
    months[(g["white_username"].lower(), d.year, d.month)].add(g["id"])
    months[(g["black_username"].lower(), d.year, d.month)].add(g["id"])

cc = {}
for (user, y, m) in list(months):
    try:
        data = fetch(f"https://api.chess.com/pub/player/{user}/games/{y}/{m:02d}")
    except Exception as e:
        print("skip", user, y, m, e)
        continue
    for rg in data.get("games", []):
        acc = rg.get("accuracies")
        if not acc:
            continue
        cc[rg["end_time"]] = acc

rows = []
for g in games:
    acc = cc.get(g["played_at"])
    if not acc:
        continue
    w, b = acc.get("white"), acc.get("black")
    if w is None or b is None:
        continue
    rows.append(dict(id=g["id"], plies=g["plies"], tw=g["time_class"], depth=g["depth"],
                     ours_w=g["white_accuracy"], ours_b=g["black_accuracy"],
                     cc_w=w, cc_b=b, acpl_w=g["white_acpl"], acpl_b=g["black_acpl"]))

print(f"matched {len(rows)} of {len(games)} games\n")
print(f"{'id':>4} {'plies':>5} {'dep':>3} {'tc':<6} {'oursW':>6} {'ccW':>6} {'dW':>6} {'oursB':>6} {'ccB':>6} {'dB':>6} {'acplW':>6} {'acplB':>6}")
dw, db = [], []
for r in rows:
    dw.append(r["ours_w"] - r["cc_w"])
    db.append(r["ours_b"] - r["cc_b"])
    print(f"{r['id']:>4} {r['plies']:>5} {r['depth']:>3} {r['tw']:<6} {r['ours_w']:>6.1f} {r['cc_w']:>6.1f} {r['ours_w']-r['cc_w']:>6.1f} {r['ours_b']:>6.1f} {r['cc_b']:>6.1f} {r['ours_b']-r['cc_b']:>6.1f} {r['acpl_w'] or 0:>6.1f} {r['acpl_b'] or 0:>6.1f}")

all_d = dw + db
print(f"\nbias (ours - chess.com): mean {statistics.mean(all_d):+.1f}  median {statistics.median(all_d):+.1f}  sd {statistics.pstdev(all_d):.1f}  min {min(all_d):+.1f} max {max(all_d):+.1f}")

# correlation of ours vs cc
def corr(xs, ys):
    mx, my = statistics.mean(xs), statistics.mean(ys)
    num = sum((x-mx)*(y-my) for x, y in zip(xs, ys))
    den = (sum((x-mx)**2 for x in xs) * sum((y-my)**2 for y in ys)) ** 0.5
    return num/den if den else float("nan")

xs = [r["ours_w"] for r in rows] + [r["ours_b"] for r in rows]
ys = [r["cc_w"] for r in rows] + [r["cc_b"] for r in rows]
print(f"pearson r = {corr(xs, ys):.3f}")

# ACPL vs cc accuracy: does ACPL explain cc accuracy better?
ac = [x for r in rows for x in ([r["acpl_w"]] if r["acpl_w"] is not None else [])]
print("acpl vs cc acc r =", round(corr(ac, [r["cc_w"] for r in rows if r["acpl_w"] is not None]), 3))

# simple linear fit cc ~ a*ours + b
n = len(xs)
mx, my = statistics.mean(xs), statistics.mean(ys)
a = sum((x-mx)*(y-my) for x, y in zip(xs, ys)) / sum((x-mx)**2 for x in xs)
b = my - a*mx
resid = [y - (a*x+b) for x, y in zip(xs, ys)]
print(f"fit cc = {a:.3f}*ours + {b:+.2f}   resid sd {statistics.pstdev(resid):.2f}")
json.dump(dict(rows=rows, a=a, b=b), open("/Users/suze/Projects/Programming/rust/chenal/tmp/parity.json", "w"), indent=1)
