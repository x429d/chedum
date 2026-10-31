/**
 * Chedum brand mark: the same king silhouette as the app icon, punched out of the
 * teal tile that `.brand-mark` paints behind it.
 */
const KING_CROSS = "M20.9 5.2h3.2v1.9h3.4v3h-3.4V14.6h-3.2v-4.5h-3.4v-3h3.4z";
const KING_ORB =
  "M22.5 25s4.5-7.5 3-10.5c0 0-1-2.5-3-2.5s-3 2.5-3 2.5c-1.5 3 3 10.5 3 10.5";
const KING_BODY =
  "M11.5 37c5.5 3.5 15.5 3.5 21 0v-7s9-4.5 6-10.5c-4-6.5-13.5-3.5-16 4V27v-3.5c-3.5-7.5-13-10.5-16-4-3 6 5 10 5 10z";

export function BrandMark(props: { size?: number }) {
  const size = () => props.size ?? 20;
  return (
    <svg viewBox="0 0 45 45" width={size()} height={size()} aria-hidden="true">
      <g fill="currentColor">
        <path d={KING_CROSS} />
        <path d={KING_ORB} />
        <path d={KING_BODY} />
      </g>
    </svg>
  );
}
