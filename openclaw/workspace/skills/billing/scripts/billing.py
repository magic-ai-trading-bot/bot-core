#!/usr/bin/env python3
"""OpenClaw billing reporter – parse session JSONL and report costs."""

import glob
import json
import os
import sys
from collections import defaultdict
from datetime import datetime, timedelta, timezone

# OpenClaw stores sessions here (inside container: /home/node/.openclaw/...)
SESSIONS_DIR = os.path.expanduser("~/.openclaw/agents/main/sessions")
VALID_CMDS = ["today", "week", "month", "7d", "30d", "total", "models"]


# ── helpers ──────────────────────────────────────────────────────────

def fmt_cost(v):
    if 0 < abs(v) < 0.01:
        return f"${v:,.3f}"
    return f"${v:,.2f}"


def fmt_tokens(n):
    n = int(n)
    if n >= 1_000_000:
        return f"{n / 1_000_000:.1f}M"
    if n >= 1_000:
        return f"{n / 1_000:.1f}K"
    return str(n)


def fmt_io_ratio(input_tokens, output_tokens):
    if output_tokens <= 0:
        return f"{fmt_tokens(input_tokens)} in (output ≈ 0)"
    ratio = input_tokens / output_tokens
    if ratio >= 1:
        return f"{ratio:.0f}:1"
    return f"1:{1 / ratio:.0f}"


def pct(a, b):
    if b == 0:
        return 0.0
    return (a - b) / b * 100


def safe_div(a, b, default=0.0):
    return a / b if b else default


def date_str(dt):
    return dt.strftime("%b %d")


def trend_arrow(change_pct):
    if change_pct > 1:
        return f"\u2191 {abs(change_pct):.1f}%"
    if change_pct < -1:
        return f"\u2193 {abs(change_pct):.1f}%"
    return "~ flat"


# ── parsing ──────────────────────────────────────────────────────────

def parse_sessions():
    pattern = os.path.join(SESSIONS_DIR, "*.jsonl")
    files = glob.glob(pattern)
    if not files:
        return []

    turns = []
    for fpath in files:
        basename = os.path.basename(fpath)
        if ".deleted." in basename or not basename.endswith(".jsonl"):
            continue
        session_id = basename.removesuffix(".jsonl")
        try:
            with open(fpath, "r", encoding="utf-8", errors="replace") as f:
                prev_user_text = ""
                for line in f:
                    line = line.strip()
                    if not line:
                        continue
                    try:
                        obj = json.loads(line)
                    except json.JSONDecodeError:
                        continue

                    if obj.get("type") != "message":
                        continue
                    msg = obj.get("message", {})
                    role = msg.get("role")

                    if role == "user":
                        parts = msg.get("content", [])
                        if isinstance(parts, list):
                            prev_user_text = " ".join(
                                p.get("text", "") for p in parts if isinstance(p, dict)
                            )
                        elif isinstance(parts, str):
                            prev_user_text = parts
                        continue

                    if role != "assistant":
                        continue

                    usage = msg.get("usage", {})
                    cost_obj = usage.get("cost", {})
                    cost_total = cost_obj.get("total", 0) if isinstance(cost_obj, dict) else 0
                    if cost_total <= 0:
                        continue

                    ts_raw = obj.get("timestamp", "")
                    try:
                        ts = datetime.fromisoformat(ts_raw.replace("Z", "+00:00"))
                    except (ValueError, AttributeError):
                        epoch = msg.get("timestamp")
                        if epoch:
                            ts = datetime.fromtimestamp(epoch / 1000, tz=timezone.utc)
                        else:
                            continue

                    assistant_text = ""
                    parts = msg.get("content", [])
                    if isinstance(parts, list):
                        assistant_text = " ".join(
                            p.get("text", "") for p in parts if isinstance(p, dict)
                        )
                    is_hb = (
                        "HEARTBEAT" in prev_user_text.upper()
                        or assistant_text.strip() == "HEARTBEAT_OK"
                    )

                    turns.append({
                        "ts": ts,
                        "date": ts.date(),
                        "model": msg.get("model", "unknown"),
                        "provider": msg.get("provider", "unknown"),
                        "cost": cost_total,
                        "cost_input": cost_obj.get("input", 0) if isinstance(cost_obj, dict) else 0,
                        "cost_output": cost_obj.get("output", 0) if isinstance(cost_obj, dict) else 0,
                        "cost_cache_read": cost_obj.get("cacheRead", 0) if isinstance(cost_obj, dict) else 0,
                        "cost_cache_write": cost_obj.get("cacheWrite", 0) if isinstance(cost_obj, dict) else 0,
                        "input_tokens": usage.get("input", 0),
                        "output_tokens": usage.get("output", 0),
                        "cache_read": usage.get("cacheRead", 0),
                        "cache_write": usage.get("cacheWrite", 0),
                        "is_heartbeat": is_hb,
                        "session_id": session_id,
                    })
                    prev_user_text = ""
        except OSError:
            continue

    turns.sort(key=lambda t: t["ts"])
    return turns


# ── aggregation ──────────────────────────────────────────────────────

def aggregate(turns):
    if not turns:
        return None
    total_cost = sum(t["cost"] for t in turns)
    total_input = sum(t["input_tokens"] for t in turns)
    total_output = sum(t["output_tokens"] for t in turns)
    total_cache_read = sum(t["cache_read"] for t in turns)
    total_cache_write = sum(t["cache_write"] for t in turns)
    n = len(turns)
    hb_turns = [t for t in turns if t["is_heartbeat"]]
    user_turns = [t for t in turns if not t["is_heartbeat"]]

    model_cost = defaultdict(lambda: {"cost": 0, "turns": 0})
    for t in turns:
        model_cost[t["model"]]["cost"] += t["cost"]
        model_cost[t["model"]]["turns"] += 1
    top_model = max(model_cost, key=lambda m: model_cost[m]["cost"]) if model_cost else "N/A"

    sess_cost = defaultdict(float)
    for t in turns:
        sess_cost[t["session_id"]] += t["cost"]
    top_session = max(sess_cost, key=sess_cost.get) if sess_cost else "N/A"

    first_date = turns[0]["date"]
    last_date = turns[-1]["date"]
    days = max((last_date - first_date).days + 1, 1)

    cache_total = total_cache_read + total_cache_write + total_input + total_output
    cache_rate = safe_div(total_cache_read, cache_total) * 100

    cache_savings = 0.0
    for t in turns:
        if t["cache_read"] > 0 and t["input_tokens"] > 0:
            input_rate = t["cost_input"] / t["input_tokens"]
            cache_savings += t["cache_read"] * input_rate - t["cost_cache_read"]

    sorted_sessions = sorted(sess_cost.items(), key=lambda x: x[1], reverse=True)
    top_3_sessions = sorted_sessions[:3]

    return {
        "cost": total_cost,
        "turns": n,
        "input_tokens": total_input,
        "output_tokens": total_output,
        "cache_read": total_cache_read,
        "cache_write": total_cache_write,
        "avg_per_turn": safe_div(total_cost, n),
        "top_model": top_model,
        "top_model_cost": model_cost[top_model]["cost"] if top_model != "N/A" else 0,
        "hb_turns": len(hb_turns),
        "hb_cost": sum(t["cost"] for t in hb_turns),
        "user_turns": len(user_turns),
        "user_cost": sum(t["cost"] for t in user_turns),
        "cache_rate": cache_rate,
        "token_ratio": safe_div(total_output, total_input),
        "top_session": top_session,
        "top_session_cost": sess_cost.get(top_session, 0),
        "days": days,
        "avg_daily": safe_div(total_cost, days),
        "first_date": first_date,
        "last_date": last_date,
        "model_cost": dict(model_cost),
        "avg_per_user_turn": safe_div(sum(t["cost"] for t in user_turns), len(user_turns)),
        "cost_per_1k_out": safe_div(total_cost, total_output) * 1000,
        "output_per_dollar": safe_div(total_output, total_cost),
        "cache_savings": cache_savings,
        "session_count": len(sess_cost),
        "avg_session_cost": safe_div(total_cost, len(sess_cost)),
        "avg_input_per_user_turn": safe_div(sum(t["input_tokens"] for t in user_turns), len(user_turns)),
        "top_3_sessions": top_3_sessions,
    }


# ── filters ──────────────────────────────────────────────────────────

def now_utc():
    return datetime.now(timezone.utc)


def filter_today(turns):
    today = now_utc().date()
    return [t for t in turns if t["date"] == today]


def filter_week(turns):
    today = now_utc().date()
    monday = today - timedelta(days=today.weekday())
    return [t for t in turns if monday <= t["date"] <= today]


def filter_prev_week(turns):
    today = now_utc().date()
    monday = today - timedelta(days=today.weekday())
    prev_monday = monday - timedelta(days=7)
    prev_sunday = monday - timedelta(days=1)
    return [t for t in turns if prev_monday <= t["date"] <= prev_sunday]


def filter_month(turns):
    today = now_utc().date()
    first = today.replace(day=1)
    return [t for t in turns if first <= t["date"] <= today]


def filter_prev_month(turns):
    today = now_utc().date()
    first = today.replace(day=1)
    prev_last = first - timedelta(days=1)
    prev_first = prev_last.replace(day=1)
    return [t for t in turns if prev_first <= t["date"] <= prev_last]


def filter_last_n_days(turns, n):
    today = now_utc().date()
    start = today - timedelta(days=n - 1)
    return [t for t in turns if start <= t["date"] <= today]


def filter_prev_n_days(turns, n):
    today = now_utc().date()
    end = today - timedelta(days=n)
    start = end - timedelta(days=n - 1)
    return [t for t in turns if start <= t["date"] <= end]


# ── formatters ───────────────────────────────────────────────────────

def fmt_period(agg, label, prev_agg=None):
    lines = [f"  Cost: {fmt_cost(agg['cost'])} \u00b7 Turns: {agg['turns']:,}"]
    if prev_agg:
        change = pct(agg["cost"], prev_agg["cost"])
        lines[0] += f" ({trend_arrow(change)} vs prev)"
    lines.append(f"  Avg: {fmt_cost(agg['avg_per_user_turn'])}/user turn \u00b7 Top: {agg['top_model']}")
    user_pct = safe_div(agg['user_cost'], agg['cost']) * 100
    hb_pct = safe_div(agg['hb_cost'], agg['cost']) * 100
    lines.append(
        f"  Productive: {fmt_cost(agg['user_cost'])} ({user_pct:.0f}%) \u00b7 "
        f"Heartbeat: {fmt_cost(agg['hb_cost'])} ({hb_pct:.0f}%)"
    )
    lines.append(
        f"  Tokens: {fmt_tokens(agg['input_tokens'])} in / {fmt_tokens(agg['output_tokens'])} out"
    )
    lines.append(f"  Avg context: {fmt_tokens(agg['avg_input_per_user_turn'])} tokens/user turn")
    cache_r = agg["cache_rate"]
    if cache_r > 0:
        cache_line = f"  Cache hit: {cache_r:.1f}%"
        if agg["cache_savings"] > 0:
            cache_line += f" \u00b7 Saved ~{fmt_cost(agg['cache_savings'])}"
        lines.append(cache_line)
    lines.append(
        f"  Efficiency: {fmt_cost(agg['cost_per_1k_out'])}/1K out \u00b7 "
        f"Output per $1: {fmt_tokens(agg['output_per_dollar'])}"
    )
    return "\n".join(lines)


def bar_chart(turns, n_days=14):
    today = now_utc().date()
    start = today - timedelta(days=n_days - 1)
    daily = defaultdict(float)
    for t in turns:
        if start <= t["date"] <= today:
            daily[t["date"]] += t["cost"]

    if not daily:
        return "  (no data)"

    max_cost = max(daily.values()) if daily else 1
    bar_width = 20
    lines = []
    d = start
    while d <= today:
        c = daily.get(d, 0)
        fill = int(round(c / max_cost * bar_width)) if max_cost > 0 else 0
        bar = "\u2588" * fill if fill > 0 else "\u258f"
        lines.append(f"{d.strftime('%b %d')} {bar} {fmt_cost(c)}")
        d += timedelta(days=1)
    return "\n".join(lines)


def nav_footer(exclude=None):
    cmds = [("models", "/billing models"), ("today", "/billing today"),
            ("7d", "/billing 7d"), ("30d", "/billing 30d"), ("total", "/billing total")]
    items = [c[1] for c in cmds if c[0] != exclude]
    return "\n" + "\u2501" * 20 + "\n" + " \u00b7 ".join(items) + "\n\u26a0 Estimated from token counts"


def weekly_chart(turns, max_weeks=12):
    if not turns:
        return "  (no data)"
    weekly = defaultdict(float)
    week_ranges = {}
    for t in turns:
        iso_year, iso_week, _ = t["date"].isocalendar()
        key = (iso_year, iso_week)
        weekly[key] += t["cost"]
        if key not in week_ranges:
            week_ranges[key] = [t["date"], t["date"]]
        else:
            if t["date"] < week_ranges[key][0]:
                week_ranges[key][0] = t["date"]
            if t["date"] > week_ranges[key][1]:
                week_ranges[key][1] = t["date"]
    if not weekly:
        return "  (no data)"
    sorted_weeks = sorted(weekly.keys())
    if len(sorted_weeks) > max_weeks:
        sorted_weeks = sorted_weeks[-max_weeks:]
    max_cost = max(weekly[w] for w in sorted_weeks)
    bar_width = 20
    lines = []
    for year, week in sorted_weeks:
        cost = weekly[(year, week)]
        dates = week_ranges[(year, week)]
        fill = int(round(cost / max_cost * bar_width)) if max_cost > 0 else 0
        bar = "\u2588" * fill if fill > 0 else "\u258f"
        start_str = dates[0].strftime("%b%d")
        end_str = dates[1].strftime("%b%d")
        lines.append(f"W{week:02d} {start_str}-{end_str} {bar} {fmt_cost(cost)}")
    return "\n".join(lines)


def projected_monthly(turns):
    recent = filter_last_n_days(turns, 7)
    if not recent:
        return None
    daily = defaultdict(float)
    for t in recent:
        daily[t["date"]] += t["cost"]
    if not daily:
        return None
    avg = sum(daily.values()) / len(daily)
    return avg * 30


# ── subcommands ──────────────────────────────────────────────────────

def cmd_dashboard(turns):
    today = now_utc()
    out = ["\U0001f4b0 Billing Dashboard", "\u2501" * 20, ""]

    t_today = filter_today(turns)
    if t_today:
        a = aggregate(t_today)
        out.append(f"\U0001f4c5 Today ({date_str(today)})")
        out.append(f"  Cost: {fmt_cost(a['cost'])} \u00b7 Turns: {a['turns']:,}")
        out.append(f"  Top: {a['top_model']}")
    else:
        out.append(f"\U0001f4c5 Today ({date_str(today)})")
        out.append("  No activity yet")
    out.append("")

    t_7d = filter_last_n_days(turns, 7)
    p_7d = filter_prev_n_days(turns, 7)
    if t_7d:
        a7 = aggregate(t_7d)
        ap7 = aggregate(p_7d) if p_7d else None
        out.append("\U0001f4ca Last 7 Days")
        out.append(fmt_period(a7, "7d", ap7))
    out.append("")

    t_30d = filter_last_n_days(turns, 30)
    p_30d = filter_prev_n_days(turns, 30)
    if t_30d:
        a30 = aggregate(t_30d)
        ap30 = aggregate(p_30d) if p_30d else None
        out.append("\U0001f4c8 Last 30 Days")
        out.append(fmt_period(a30, "30d", ap30))
    out.append("")

    a_all = aggregate(turns)
    if a_all:
        out.append("\U0001f3e6 All Time")
        out.append(
            f"  Cost: {fmt_cost(a_all['cost'])} \u00b7 "
            f"Since: {date_str(a_all['first_date'])} ({a_all['days']} days)"
        )
        out.append(f"  Avg: {fmt_cost(a_all['avg_daily'])}/day")
        out.append(f"  Sessions: {a_all['session_count']} \u00b7 Avg: {fmt_cost(a_all['avg_session_cost'])}/session")
    out.append("")

    out.append("\U0001f4c9 Daily (14d)")
    out.append(bar_chart(turns, 14))
    out.append("")

    proj = projected_monthly(turns)
    if proj is not None and a_all:
        out.append("\U0001f4a1 Projection (based on last 7d)")
        recent_7d = filter_last_n_days(turns, 7)
        daily_costs = defaultdict(float)
        for t in recent_7d:
            daily_costs[t["date"]] += t["cost"]
        recent_avg = sum(daily_costs.values()) / len(daily_costs) if daily_costs else 0
        out.append(f"  Recent avg: {fmt_cost(recent_avg)}/day")
        out.append(f"  Estimated: ~{fmt_cost(proj)}/mo")
        alltime_monthly = a_all["avg_daily"] * 30
        out.append(f"  vs all-time avg: ~{fmt_cost(alltime_monthly)}/mo")

    out.append(nav_footer())
    return "\n".join(out)


def cmd_period(turns, label, filter_fn, prev_filter_fn=None):
    filtered = filter_fn(turns)
    if not filtered:
        return f"No data for: {label}"
    agg = aggregate(filtered)
    prev_agg = None
    if prev_filter_fn:
        prev = prev_filter_fn(turns)
        if prev and len(prev) >= 3:
            pa = aggregate(prev)
            if pa and pa["days"] >= 3:
                prev_agg = pa

    out = [f"\U0001f4ca {label}", "\u2501" * 20, ""]
    out.append(fmt_period(agg, label, prev_agg))
    out.append("")
    out.append("  Top sessions:")
    for sid, cost in agg["top_3_sessions"]:
        out.append(f"    {sid[:8]}... {fmt_cost(cost)}")
    out.append(f"  Sessions: {agg['session_count']} \u00b7 Avg: {fmt_cost(agg['avg_session_cost'])}/session")
    out.append(f"  Input:Output ratio: {fmt_io_ratio(agg['input_tokens'], agg['output_tokens'])}")
    return "\n".join(out)


def cmd_today(turns):
    today = now_utc()
    result = cmd_period(turns, f"Today ({date_str(today)})", filter_today)
    return result + nav_footer(exclude="today")


def cmd_week(turns):
    today = now_utc().date()
    monday = today - timedelta(days=today.weekday())
    sunday = monday + timedelta(days=6)
    days_in = today.weekday() + 1
    label = f"This Week ({date_str(monday)} – {date_str(sunday)}, day {days_in}/7)"
    prev_fn = filter_prev_week if days_in >= 5 else None
    return cmd_period(turns, label, filter_week, prev_fn) + nav_footer(exclude="week")


def cmd_month(turns):
    return cmd_period(turns, "This Month", filter_month, filter_prev_month) + nav_footer(exclude="month")


def cmd_7d(turns):
    result = cmd_period(
        turns, "Last 7 Days",
        lambda t: filter_last_n_days(t, 7),
        lambda t: filter_prev_n_days(t, 7),
    )
    return result + nav_footer(exclude="7d")


def cmd_30d(turns):
    result = cmd_period(
        turns, "Last 30 Days",
        lambda t: filter_last_n_days(t, 30),
        lambda t: filter_prev_n_days(t, 30),
    )
    filtered = filter_last_n_days(turns, 30)
    if filtered:
        result += "\n\n\U0001f4c9 Weekly (30d)\n" + weekly_chart(filtered, max_weeks=5)
    return result + nav_footer(exclude="30d")


def cmd_total(turns):
    agg = aggregate(turns)
    if not agg:
        return "No data."
    out = ["\U0001f3e6 All Time", "\u2501" * 20, ""]
    out.append(fmt_period(agg, "all"))
    out.append("")
    out.append(f"  Period: {date_str(agg['first_date'])} \u2013 {date_str(agg['last_date'])} ({agg['days']} days)")
    out.append(f"  Daily avg: {fmt_cost(agg['avg_daily'])}")
    out.append("  Top sessions:")
    for sid, cost in agg["top_3_sessions"]:
        out.append(f"    {sid[:8]}... {fmt_cost(cost)}")
    out.append(f"  Sessions: {agg['session_count']} \u00b7 Avg: {fmt_cost(agg['avg_session_cost'])}/session")
    out.append(f"  Input:Output ratio: {fmt_io_ratio(agg['input_tokens'], agg['output_tokens'])}")
    out.append("")
    out.append("\U0001f4c9 Weekly")
    out.append(weekly_chart(turns, max_weeks=12))
    out.append("")
    proj = projected_monthly(turns)
    if proj is not None:
        out.append(f"\U0001f4a1 Projected: ~{fmt_cost(proj)}/mo")
    out.append(nav_footer(exclude="total"))
    return "\n".join(out)


def cmd_models(turns):
    if not turns:
        return "No data."
    model_data = defaultdict(lambda: {"cost": 0, "turns": 0, "input": 0, "output": 0})
    for t in turns:
        m = t["model"]
        model_data[m]["cost"] += t["cost"]
        model_data[m]["turns"] += 1
        model_data[m]["input"] += t["input_tokens"]
        model_data[m]["output"] += t["output_tokens"]

    total_cost = sum(d["cost"] for d in model_data.values())
    ranked = sorted(model_data.items(), key=lambda x: x[1]["cost"], reverse=True)
    max_cost = ranked[0][1]["cost"] if ranked else 1
    bar_width = 16

    out = ["\U0001f4ca Models (All Time)", "\u2501" * 20, ""]
    for i, (model, d) in enumerate(ranked, 1):
        share = safe_div(d["cost"], total_cost) * 100
        avg = safe_div(d["cost"], d["turns"])
        fill = int(round(d["cost"] / max_cost * bar_width)) if max_cost > 0 else 0
        bar = "\u2588" * fill if fill > 0 else "\u258f"
        out.append(f"{i}. {model}")
        out.append(f"   {bar} {fmt_cost(d['cost'])} ({share:.1f}%)")
        cost_per_1k = safe_div(d["cost"], d["output"]) * 1000
        out.append(f"   {d['turns']:,} turns \u00b7 {fmt_cost(avg)}/turn \u00b7 {fmt_cost(cost_per_1k)}/1K out")
        out.append(f"   {fmt_tokens(d['input'])} in / {fmt_tokens(d['output'])} out")
        out.append("")

    out.append(nav_footer(exclude="models"))
    return "\n".join(out)


# ── main ─────────────────────────────────────────────────────────────

def main():
    if not os.path.isdir(SESSIONS_DIR):
        print("No session directory found. Is OpenClaw configured?")
        sys.exit(0)

    turns = parse_sessions()
    if not turns:
        print("No billing data found in session files.")
        sys.exit(0)

    cmd = sys.argv[1].lower() if len(sys.argv) > 1 else ""

    dispatch = {
        "": cmd_dashboard,
        "today": cmd_today,
        "week": cmd_week,
        "month": cmd_month,
        "7d": cmd_7d,
        "30d": cmd_30d,
        "total": cmd_total,
        "models": cmd_models,
    }

    if cmd not in dispatch:
        print(f"Unknown command: {cmd}")
        print(f"Valid options: {', '.join(VALID_CMDS)}")
        print("Or run with no arguments for the dashboard.")
        sys.exit(0)

    print(dispatch[cmd](turns))


if __name__ == "__main__":
    main()
