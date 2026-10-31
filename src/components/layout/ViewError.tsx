import { t } from "../../i18n";

export function ViewError(props: { error: unknown; reset: () => void }) {
  const details = (): string => {
    if (props.error instanceof Error) return props.error.stack ?? props.error.message;
    if (typeof props.error === "string") return props.error;
    try {
      return JSON.stringify(props.error);
    } catch {
      return String(props.error);
    }
  };
  return (
    <div class="empty-state" style={{ "min-height": "60vh" }} role="alert">
      <div class="empty-icon">
        <span aria-hidden="true">!</span>
      </div>
      <div style={{ "font-weight": "600", color: "var(--text)" }}>{t("app.unexpectedError")}</div>
      <button type="button" class="btn btn-primary btn-sm" onClick={() => props.reset()}>
        {t("app.reloadView")}
      </button>
      <details style={{ "max-width": "720px", width: "100%" }}>
        <summary class="dim">{t("app.errorDetails")}</summary>
        <pre class="mono" style={{ "white-space": "pre-wrap", "font-size": "10.5px", "text-align": "left" }}>
          {details()}
        </pre>
      </details>
    </div>
  );
}
