import json
import sqlite3
import subprocess
import sys
import time

DB = "/Users/suze/Library/Application Support/com.chenal.studio/chenal.db"
SF = "/Users/suze/Projects/Programming/rust/chenal/src-tauri/binaries/stockfish-aarch64-apple-darwin"
GAME = int(sys.argv[1]) if len(sys.argv) > 1 else 843
DEPTHS = [int(x) for x in (sys.argv[2].split(",") if len(sys.argv) > 2 else ["14", "18"])]


class Uci:
    def __init__(self):
        self.p = subprocess.Popen([SF], stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, bufsize=1)
        self.send("uci")
        self.wait_for("uciok")
        self.send("setoption name Threads value 6")
        self.send("setoption name Hash value 512")
        self.send("setoption name MultiPV value 1")
        self.send("isready")
        self.wait_for("readyok")

    def send(self, cmd):
        self.p.stdin.write(cmd + "\n")
        self.p.stdin.flush()

    def wait_for(self, token):
        while True:
            line = self.p.stdout.readline()
            if not line:
                raise RuntimeError("engine died")
            if line.strip() == token or line.strip().startswith(token):
                return line.strip()

    def search(self, fen, depth, searchmoves=None):
        self.send("position fen " + fen)
        cmd = f"go depth {depth}"
        if searchmoves:
            cmd += " searchmoves " + searchmoves
        self.send(cmd)
        best, score, pv = None, None, []
        while True:
            line = self.p.stdout.readline().strip()
            if line.startswith("info") and " score " in line:
                parts = line.split()
                if "cp" in parts:
                    i = parts.index("cp")
                    score = ("cp", int(parts[i + 1]))
                elif "mate" in parts:
                    i = parts.index("mate")
                    score = ("mate", int(parts[i + 1]))
                if "depth" in parts and int(parts[parts.index("depth") + 1]) != depth:
                    score = score  # keep last
            elif line.startswith("bestmove"):
                best = line.split()[1]
                return best, score

    def close(self):
        self.send("quit")


def winpct(kind, value):
    if kind == "mate":
        if value == 0:
            return 0.0
        d = min(abs(value), 20)
        return 100.0 - d * 0.2 if value > 0 else d * 0.2
    return 50 + 50 * (2 / (1 + pow(2.718281828459045, -0.00368208 * value)) - 1)


def move_acc(delta):
    raw = 103.1668100711649 * pow(2.718281828459045, -0.04354415386753951 * max(0.0, delta)) - 3.166924740191411
    return min(100.0, max(0.0, raw))


con = sqlite3.connect(DB)
con.row_factory = sqlite3.Row
rows = con.execute(
    "select ply, fen_before, fen_after, played_uci, played_san, is_critical, classification from move_evaluations where game_id=? order by ply",
    (GAME,),
).fetchall()

out = {}
eng = Uci()
for depth in DEPTHS:
    t0 = time.time()
    recs = []
    for r in rows:
        # best line in position before the move
        _, best_score = eng.search(r["fen_before"], depth)
        best_win = winpct(*best_score)
        # eval of the resulting position (current pipeline)
        _, after_score = eng.search(r["fen_after"], depth)
        after_win_player = 100.0 - winpct(*after_score)
        # eval of the played move itself at the same depth
        _, played_score = eng.search(r["fen_before"], depth, r["played_uci"])
        played_win = winpct(*played_score)
        recs.append(
            dict(
                ply=r["ply"],
                cls=r["classification"],
                crit=r["is_critical"],
                acc_after=move_acc(best_win - after_win_player),
                acc_sm=move_acc(best_win - played_win),
                d_after=best_win - after_win_player,
                d_sm=best_win - played_win,
            )
        )
    def agg(key):
        w = [(x[key], 1.5 if x["crit"] else 1.0) for x in recs]
        return sum(a * k for a, k in w) / sum(k for _, k in w)
    out[depth] = dict(
        acc_after=agg("acc_after"),
        acc_searchmoves=agg("acc_sm"),
        mean_delta_after=sum(max(0, x["d_after"]) for x in recs) / len(recs),
        mean_delta_sm=sum(max(0, x["d_sm"]) for x in recs) / len(recs),
        seconds=round(time.time() - t0, 1),
        recs=recs,
    )
    print(depth, json.dumps({k: v for k, v in out[depth].items() if k != "recs"}), flush=True)
eng.close()
json.dump(out, open(f"/Users/suze/Projects/Programming/rust/chenal/tmp/depth-{GAME}.json", "w"), indent=1)
