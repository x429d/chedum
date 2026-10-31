import math
import os
import sqlite3
import statistics
from pathlib import Path

DB = os.environ.get(
    "CHENAL_DB",
    str(Path.home() / "Library/Application Support/com.chenal.studio/chenal.db"),
)
con = sqlite3.connect(DB)
con.row_factory = sqlite3.Row


def winpct(cp):
    return 50 + 50 * (2 / (1 + math.exp(-0.00368208 * cp)) - 1)


def move_acc(delta, bonus=0.0):
    raw = 103.1668100711649 * math.exp(-0.04354415386753951 * delta) - 3.166924740191411
    return min(100.0, max(0.0, raw + bonus))


def squeeze(v, lo, hi):
    return max(lo, min(hi, v))


def lichess_game(evals, start_white=True, already_winpct=False):
    """Score a game the way Lichess does.

    evals holds white-relative centipawns per position, or win percentages when
    already_winpct is set.
    """
    wp = list(evals) if already_winpct else [winpct(e) for e in evals]
    n = len(wp)
    window = int(squeeze((n - 1) / 10, 2, 8))
    windows = [wp[:window]] * max(0, window - 2) + [wp[i:i + window] for i in range(0, n - window + 1)]
    weights = []
    for w in windows:
        sd = statistics.pstdev(w) if len(w) > 1 else 0.0
        weights.append(squeeze(sd, 0.5, 12) if sd > 0 else None)

    per_color = {0: [], 1: []}  # 0 = white, 1 = black; entries are (accuracy, weight)
    for i in range(n - 1):
        before, after = wp[i], wp[i + 1]
        color = i % 2  # even index means white is to move
        b = before if color == 0 else 100 - before
        a = after if color == 0 else 100 - after
        acc = 100.0 if a >= b else move_acc(b - a)
        per_color[color].append((acc, weights[i]))

    out = {}
    for c, rows in per_color.items():
        wrows = [(a, w) for a, w in rows if w is not None]
        wsum = sum(w for _, w in wrows)
        wmean = sum(a * w for a, w in wrows) / wsum if wsum else None
        hmean = len(rows) / sum(1 / a for a, _ in rows if a > 0) if rows else None
        out[c] = (wmean + hmean) / 2 if wmean is not None and hmean is not None else None
    return out


def variants(game_id):
    rows = con.execute(
        "select ply, win_prob_loss, move_accuracy, is_critical, win_prob_white from move_evaluations where game_id=? order by ply",
        (game_id,),
    ).fetchall()
    w = [(r["move_accuracy"], 1.5 if r["is_critical"] else 1.0) for r in rows if r["ply"] % 2 == 1]
    b = [(r["move_accuracy"], 1.5 if r["is_critical"] else 1.0) for r in rows if r["ply"] % 2 == 0]
    flat = lambda xs: sum(a * w for a, w in xs) / sum(w for _, w in xs)
    plain = lambda xs: statistics.mean(a for a, _ in xs)
    harm = lambda xs: len(xs) / sum(1 / a for a, _ in xs if a > 0)
    return dict(
        n=len(rows),
        flat_w15=(flat(w), flat(b)),
        flat=(plain(w), plain(b)),
        harmonic=(harm(w), harm(b)),
        stored=(con.execute("select white_accuracy, black_accuracy from game_analyses where game_id=?", (game_id,)).fetchone()),
    )


# Lichess scoring from the stored white-relative win%: centipawns aren't stored
# per position, so approximate them by inverting the sigmoid.
def inv(cp_like):
    # Win percentage -> centipawns.
    p = (cp_like - 50) / 50
    if abs(p) >= 1:
        p = math.copysign(0.999999, p)
    return math.log((1 + p) / (1 - p)) / 0.00368208


def lichess_from_db(game_id):
    rows = con.execute(
        "select ply, win_prob_white, eval_mate, eval_cp from move_evaluations where game_id=? order by ply",
        (game_id,),
    ).fetchall()
    # Position 0's win% isn't stored; rebuild it from the first move's before-state.
    evals = []
    first = con.execute(
        "select win_prob_white, win_prob_loss from move_evaluations where game_id=? order by ply limit 1", (game_id,)
    ).fetchone()
    before0 = first["win_prob_white"] + first["win_prob_loss"] * (1 if True else 1)
    evals.append(min(99.9, before0))
    evals += [r["win_prob_white"] for r in rows]
    return lichess_game_wp(evals)


def lichess_game_wp(wp, start_white=True):
    return lichess_game(wp, start_white, already_winpct=True)


chesscom = {
    843: (48.4, 36.9),
    842: (90.9, 71.4),
    841: (88.9, 87.3),
    839: (74.4, 69.8),
}

for gid in (843, 842, 841, 839, 840, 838):
    v = variants(gid)
    lich = lichess_from_db(gid)
    cc = chesscom.get(gid, ("?", "?"))
    print(f"game {gid}  plies={v['n']}")
    print(f"  stored (chenal)   W {v['stored'][0]:.1f}  B {v['stored'][1]:.1f}")
    print(f"  flat w=1.5        W {v['flat_w15'][0]:.1f}  B {v['flat_w15'][1]:.1f}")
    print(f"  flat w=1          W {v['flat'][0]:.1f}  B {v['flat'][1]:.1f}")
    print(f"  harmonic          W {v['harmonic'][0]:.1f}  B {v['harmonic'][1]:.1f}")
    print(f"  lichess-style     W {lich[0]:.1f}  B {lich[1]:.1f}")
    print(f"  chess.com         W {cc[0]}  B {cc[1]}")
