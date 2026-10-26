export function AccuracyRing(props: { value: number | null; color: string; label: string }) {
  const radius = 44;
  const circumference = 2 * Math.PI * radius;
  const value = () => (props.value === null ? 0 : Math.max(0, Math.min(100, props.value)));
  const offset = () => circumference * (1 - value() / 100);
  return (
    <div class="ring-wrap">
      <svg viewBox="0 0 104 104" width="104" height="104">
        <circle cx="52" cy="52" r={radius} fill="none" stroke="#232a3b" stroke-width="7" />
        <circle
          cx="52"
          cy="52"
          r={radius}
          fill="none"
          stroke={props.color}
          stroke-width="7"
          stroke-linecap="round"
          stroke-dasharray={`${circumference} ${circumference}`}
          stroke-dashoffset={offset()}
          transform="rotate(-90 52 52)"
          style={{ transition: "stroke-dashoffset 0.5s cubic-bezier(0.2, 0.8, 0.2, 1)" }}
        />
      </svg>
      <div class="ring-center">
        <span class="ring-value" style={{ color: props.color }}>
          {props.value === null ? "—" : props.value.toFixed(1)}
        </span>
        <span class="ring-caption">{props.label}</span>
      </div>
    </div>
  );
}
